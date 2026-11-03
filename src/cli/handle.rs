use crate::cli::types::*;
use crate::cli::format::{args_space_separated_len, join_args, repeated_underline, format_command};
use crate::cli::config::schema::COMMANDS;
use crate::cli::hint::{command_hint, usage_hint};
use std::{path::PathBuf, collections::HashMap};

fn repeated_config_err(args: &[String], i: usize) -> String {
    let len = args_space_separated_len(args, i);
    format!(
        "argument `{}` for command `{}` already provided:\n{}\n{} -- repeated argument",
        args[i], args[0], join_args(args), " ".repeat(len) + &"~".repeat(args[i].len())
    )
}

fn missing_req_args(arg_map: &ArgumentRegistry, command: &str, args: &[ArgConfig], full_args: &str, fmt_usage: &str) -> Result<(), String> {
    let missing: Vec<&ArgConfig> = args.iter()
        .filter(|&a| a.required && arg_map.get(a.config).is_some_and(|arg| !arg.visited))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Missing required arguments for command `{}`:\n\t{}\n{}\n{}\n{} out of {} required fields missing.\n\nCommand usage:\n\t{}",
            command,
            missing.iter().map(|arg| 
                if let Some(val) = arg.value { format!("{} <{}>", arg.config, val.as_str()) }
                else { arg.config.to_string() }
            ).collect::<Vec<_>>().join("\n\t"),
            full_args, "~".repeat(full_args.len()), missing.len(), args.iter().filter(|&a| a.required).count(), fmt_usage
        ));
    }
    Ok(())
}

fn parse_value(value_type: ValueType, argument: &str, val_arg: &str) -> Result<Value, String> {
    let val_str_rep = value_type.as_str();
    let fmt_err_usage = format!(
        "Could not parse <{}> from `{}`.\nArgument `{}` expected <{}>, got `{}`.",
        val_str_rep, val_arg, argument, val_str_rep, val_arg
    );
    match value_type {
        ValueType::UInt => match val_arg.parse::<usize>() {
            Ok(i) => Ok(Value::UInt(i)),
            Err(_) => Err(fmt_err_usage),
        },
        ValueType::Path => match val_arg.parse::<PathBuf>() {
            Ok(p) => Ok(Value::Path(p)),
            Err(_) => Err(fmt_err_usage),
        }
    }
}

fn handle_value<'a>(expect: &ParseState<'a>, args: &'a[String], i: usize, full_args: &str) -> Result<Value, String> {
    let ParseState::ExpectingValue {
        value_type: expect_val,
        argument: expect_val_for
    } = expect else {
        return Err(String::from("Program Error: `handle_value` is called in the wrong parsing state.\nState is `ExpectingArgument`; expected `ExpectingValue`"));
    };
    let arg = args[i].as_str();
    match parse_value(*expect_val, expect_val_for, arg) {
        Err(err) => Err(format!(
                        "{}\n{}\n{} -- parse error",
                        err, full_args, " ".repeat(args_space_separated_len(args, i)) + &repeated_underline(arg)
                    )),
        Ok(val) => Ok(val),
    }
}

fn handle_argument<'a>(arg_map: &mut ArgumentRegistry, args: &'a[String], i: usize, fmt_usage: &str)
-> Result<Option<ParseState<'a>>, String> {
    let arg = args[i].as_str();
    let Some(val) = arg_map.get_mut(arg) else {
        return Err(format!("unknown argument `{}`\nExpected usage:\n{}", arg, fmt_usage));
    };
    if val.visited {
        Err(repeated_config_err(args, i))
    } else {
        val.visited = true;
        Ok(match val.value_type {
            None => None,
            Some(value) => Some(ParseState::ExpectingValue {value_type: value, argument: arg}),
        })
    }
}

fn parse_command_values<'a>(args: &'a[String], arg_map: &mut ArgumentRegistry<'a>, fmt_usage: &'a str)
-> Result<Vec<ArgValue>, String> {
    let full_args = join_args(args);
    let mut values: Vec<ArgValue> = Vec::new();
    let mut state: ParseState = ParseState::ExpectingArgument;
    for i in 1..args.len() {
        if matches!(&state, ParseState::ExpectingArgument) {
            if let Some(new_state) = handle_argument(arg_map, args, i, fmt_usage)? {
                state = new_state;
            }
            values.push(ArgValue { argument: args[i].clone(), value: None });
        } else {
            let value = handle_value(&state, args, i, &full_args)?;
            let Some(last) = values.last_mut() else {
                return Err(format!("Program Error: "));
            };
            last.value = Some(value);
            state = ParseState::ExpectingArgument;
        }
    }
    if let ParseState::ExpectingValue {value_type: expect_val, argument: expect_val_for} = &state {
        return Err(format!(
            "In command `{}`: field `{}` expected `<{}>`, got nil.\n{}\n{}",
            args[0], expect_val_for, expect_val.as_str(), full_args, " ".repeat(full_args.len() - 1) + "^~~ -- missing value"
        ));
    }
    Ok(values)
}

fn parse_command<'a>(command: &'a CommandSpec, args: &'a [String]) -> Result<Vec<ArgValue>, String> {
    let args_entered = join_args(args);
    if let Some(subargs) = command.arguments {
        let mut arg_map: ArgumentRegistry = HashMap::new();
        
        for arg in subargs.iter() {
            if arg_map.contains_key(arg.config) {
                return Err(format!(
                    "Program Error: argument `{}` already exists in arg list for command `{}`.\nNot a problem with the args passed.\nError in argument collection phase of validation.",
                    arg.config, command.command
                ));
            }
            arg_map.insert(arg.config, ArgumentState {value_type: arg.value, visited: false});
        }
        
        let arg_configs = Vec::from(subargs);
        let fmt_usage = format_command((String::from(command.command), arg_configs));
        let updated_arguments = parse_command_values(args, &mut arg_map, &fmt_usage)?;
        missing_req_args(&arg_map, &args[0], subargs, &args_entered, &fmt_usage)?;
        return Ok(updated_arguments);
    }
    else if args.len() > 1 {
        let cmd_len = args[0].len() + 1;
        return Err(
            format!("Command `{}` takes 0 arguments, got {}.\n\t{}\n\t{} -- unexpected arguments provided",
            command.command, args.len() - 1, args_entered, " ".repeat(cmd_len) + &"~".repeat(args_entered.len() - cmd_len)
        ));
    }
    Ok(Vec::new())
}

pub fn parse_arguments(args: &[String]) -> Result<ParsedCommand, String> {
    if args.is_empty() {
        return Err(format!("No arguments were passed.\n{}", usage_hint()));
    }

    let command = &args[0];
    let cmd_idx = COMMANDS.iter().position(|&x| x.command == command)
        .ok_or_else(|| format!("No command '{}' exists.\n\nAvailable commands:\n\t{}", command, command_hint()))?;

    let cmd = &COMMANDS[cmd_idx];
    let updated_arguments = parse_command(cmd, &args)?;
    Ok(ParsedCommand { command: command.clone(), updated_args: updated_arguments})
}