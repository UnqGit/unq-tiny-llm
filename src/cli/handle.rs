use crate::{cli::handle::ParseState::{ExpectingArgument, ExpectingValue}, config::Config};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ValueType {
    None,
    Path,
    UInt,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Value {
    None,
    UInt(u64),
    Path(PathBuf),
}

enum ParseState<'a> {
    ExpectingArgument,
    ExpectingValue {
        value_type: ValueType,
        argument: &'a str,
    },
}

#[derive(Clone, Copy, Debug)]
struct ArgConfig<'a> {
    config: &'a str,
    required: bool,
    value: ValueType,
}

#[derive(Clone, Copy, Debug)]
struct CommandSpec<'a> {
    command: &'a str,
    arguments: Option<&'a [ArgConfig<'a>]>,
}

#[derive(Clone, Copy, Debug)]
struct ArgumentState {
    value_type: ValueType,
    visited: bool,
}

type ArgStateTable<'a> = HashMap<&'a str, ArgumentState>;

impl ValueType {
    fn as_str(&self) -> &'static str {
        match self {
            ValueType::None => "",
            ValueType::UInt => "unsigned integer",
            ValueType::Path => "path",
        }
    }
    fn is_none(&self) -> bool {
        matches!(&self, ValueType::None)
    }
    fn is_some(&self) -> bool {
        !self.is_none()
    }
}

static PREPROCESSING_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig { config: "--vocab-size",     required: true,  value: ValueType::UInt },
    ArgConfig { config: "--dataset",        required: false, value: ValueType::Path },
    ArgConfig { config: "--min-pair-freq",  required: false, value: ValueType::UInt },
];

static COMMANDS: &[CommandSpec] = &[
    CommandSpec { command: "-pre-processing", arguments: Some(PREPROCESSING_ARGUMENTS) },
];

fn collect_commands<'a>() -> Vec<(String, Vec<ArgConfig<'a>>)> {
    COMMANDS.iter()
        .map(|argm| {
            let cmd_name = argm.command.to_string();
            let cfgs = match argm.arguments {
                Some(a) => Vec::from(a),
                None => Vec::new(),
            };
            (cmd_name, cfgs)
        }).collect()
}

fn format_arguments<'a>(args: &[ArgConfig<'a>]) -> String {
    args.iter()
        .map(|arg| {
            let a_name = arg.config;
            let fmt_val_names = if arg.value.is_none() { String::new() } else { format!(" <{}>", arg.value.as_str()) };
            match arg.required {
                false => format!("[{}{}]", a_name,  fmt_val_names),
                true => format!("{}{}", a_name, fmt_val_names),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_command(command: (String, Vec<ArgConfig>)) -> String {
    let (cmd, cfgs) = command;
    if cfgs.is_empty() {
        return cmd;
    }
    format!("{}\t{}", cmd, format_arguments(&cfgs))
}

fn format_commands<'a>(commands: &'a [(String, Vec<ArgConfig>)]) -> String {
    commands.iter().map(|f| format_command(f.clone())).collect::<Vec<_>>().join("\n")
}

fn command_hint() -> String {
    let res = collect_commands();
    format_commands(&res)
}

fn usage_hint() -> String {
    let mut result = "Usage: cargo run -- <command> <arguments>".to_string();

    let commands: Vec<String> = COMMANDS
        .iter()
        .map(|arg| arg.command.to_string())
        .collect();

    result += format!("\n\nAvailable commands:\n\t{}", commands.join("\n\t")).as_str();
    result += format!("\n\nCommand Usage:\n\t{}", command_hint()).as_str();

    result
}

fn is_first_char_alpha(s: &str) -> bool {
    s.chars().next().map_or(false, |c| c.is_ascii_alphabetic())
}

fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|s|
            if is_first_char_alpha(s) {
                format!("\"{}\"", s)
            } else {
                s.clone()
            }
        )
        .collect::<Vec<_>>()
        .join(" ")
}

fn args_space_separated_len(args: &[String], i: usize) -> usize {
    args[0..i]
        .iter()
        .fold(0, |l, s| l + s.len() + 1 + 
            if is_first_char_alpha(s) { 2 }
            else { 0 }
        )
}

fn repeated_underline(arg: &str) -> String {
    "~".repeat(arg.len() + if is_first_char_alpha(arg) { 2 } else { 0 })
}

fn repeated_config_err(args: &[String], i: usize) -> String {
    let len = args_space_separated_len(args, i);
    format!(
        "argument `{}` for command `{}` already provided:\n{}\n{} -- repeated argument",
        args[i], args[0], join_args(args), " ".repeat(len) + &"~".repeat(args[i].len())
    )
}

fn parse_value(value_type: ValueType, argument: &str, val_arg: &str) -> Result<Value, String> {
    let val_str_rep = value_type.as_str();
    let fmt_err_usage = format!(
        "Could not parse <{}> from `{}`.\nArgument `{}` expected <{}>, got `{}`.",
        val_str_rep, val_arg, argument, val_str_rep, val_arg
    );
    match value_type {
        ValueType::UInt => match val_arg.parse::<u64>() {
            Ok(i) => Ok(Value::UInt(i)),
            Err(_) => Err(fmt_err_usage),
        },
        ValueType::Path => match val_arg.parse::<PathBuf>() {
            Ok(p) => Ok(Value::Path(p)),
            Err(_) => Err(fmt_err_usage),
        }
        ValueType::None => unreachable!(),
    }
}

fn missing_req_args(arg_map: &ArgStateTable, command: &str, args: &[ArgConfig], full_args: &str, fmt_usage: &str) -> Result<(), String> {
    let missing: Vec<&ArgConfig> = args.iter()
        .filter(|&a| a.required && arg_map.get(a.config).is_some_and(|arg| !arg.visited))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Missing required arguments for command `{}`:\n\t{}\n{}\n{}\n{} out of {} required fields missing.\n\nCommand usage:\n\t{}",
            command,
            missing.iter().map(|arg| 
                if arg.value.is_some() { format!("{} <{}>", arg.config, arg.value.as_str()) }
                else { arg.config.to_string() }
            ).collect::<Vec<_>>().join("\n\t"),
            full_args, "~".repeat(full_args.len()), missing.len(), args.iter().filter(|&a| a.required).count(), fmt_usage
        ));
    }
    Ok(())
}

fn handle_value<'a>(expect: &ParseState<'a>, args: &'a[String], i: usize, full_args: &str) -> Result<(), String> {
    if let ExpectingValue {value_type: expect_val, argument: expect_val_for} = expect {
        let arg = args[i].as_str();
        let val = parse_value(*expect_val, expect_val_for, arg);
        if let Err(err) = val {
            return Err(format!(
                "{}\n{}\n{} -- parse error", err, full_args, " ".repeat(args_space_separated_len(args, i)) + &repeated_underline(arg)
            ));
        }
        // TODO: update_config
    }
    Ok(())
}

fn handle_argument<'a>(
    arg_map: &mut ArgStateTable,
    args: &'a[String], i: usize, fmt_usage: &str
) -> Result<Option<ParseState<'a>>, String> {
    let arg = args[i].as_str();
    if let Some(val) = arg_map.get_mut(arg) {
        if val.visited {
            Err(repeated_config_err(args, i))
        } else {
            val.visited = true;
            Ok(val.value_type.is_some().then_some(
                    ParseState::ExpectingValue {value_type: val.value_type, argument: arg}
            ))
        }
    } else {
        Err(format!("unknown argument `{}`\nExpected usage:\n{}", arg, fmt_usage))
    }
}

fn handle_command_values(
    args: &[String],
    arg_map: &mut ArgStateTable,
    _config: &mut Config,
    fmt_usage: &str,
) -> Result<(), String> {
    let full_args = join_args(args);
    let mut state: ParseState = ExpectingArgument;
    for i in 1..args.len() {
        if matches!(state, ExpectingArgument) {
            if let Some(new_state) = handle_argument(arg_map, args, i, fmt_usage)? {
                state = new_state;
            }
        } else {
            handle_value(&state, args, i, &full_args)?;
            state = ParseState::ExpectingArgument;
        }
    }
    if let ExpectingValue{value_type: expect_val, argument: expect_val_for} = &state {
        return Err(format!(
            "In command `{}`: field `{}` expected `<{}>`, got nil.\n{}\n{}",
            args[0], expect_val_for, expect_val.as_str(), full_args, " ".repeat(full_args.len() - 1) + "^~~ -- missing value"
        ));
    }
    Ok(())
}

fn handle_command(command: &CommandSpec, args: &[String], config: &mut Config) -> Result<(), String> {
    let args_entered = join_args(args);
    if let Some(subargs) = command.arguments {
        let mut arg_map: ArgStateTable = HashMap::new();
        
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
        handle_command_values(args, &mut arg_map, config, &fmt_usage)?;
        missing_req_args(&arg_map, &args[0], subargs, &args_entered, &fmt_usage)?;
    }
    else if args.len() > 1 {
        let cmd_len = args[0].len() + 1;
        return Err(
            format!("Command `{}` takes 0 arguments, got {}.\n\t{}\n\t{} -- unexpected arguments provided",
            command.command, args.len() - 1, args_entered, " ".repeat(cmd_len) + &"~".repeat(args_entered.len() - cmd_len)
        ));
    }
    Ok(())
}

pub fn handle_args(args: &[String], def_config: &Config) -> Result<Config, String> {
    if args.is_empty() {
        return Err(usage_hint());
    }

    let mut config = def_config.clone();
    let command = &args[0];
    let cmd_idx = COMMANDS.iter().position(|&x| x.command == command)
        .ok_or_else(|| format!("No command '{}' exists.\n\nAvailable commands:\n\t{}", command, command_hint()))?;

    let cmd = &COMMANDS[cmd_idx];
    handle_command(cmd, &args, &mut config)?;
    Ok(config)
}