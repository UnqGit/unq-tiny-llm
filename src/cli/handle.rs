#![allow(warnings)] // for this TU for getting clean output in cli for debugging
use crate::config::Config;
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Copy, Debug)]
enum Field<T = &'static str> {
    Optional(T),
    Required(T),
}
use Field::{Optional, Required};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ValueType {
    None,
    Path,
    Integer,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Value {
    None(),
    Integer(i64),
    Path(PathBuf),
}

#[derive(Clone, Copy, Debug)]
struct ArgConfig<'a> {
    config: Field<&'a str>,
    value: ValueType,
}

#[derive(Clone, Copy, Debug)]
struct Argument<'a> {
    command: &'a str,
    indir: Option<usize>,
}

static COMMANDS: &[Argument] = &[
    Argument { command: "-pre-processing", indir: Some(0usize) },
];

static PREPROCESSING_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig { config: Required("--vocab-size"), value: ValueType::Integer },
    ArgConfig { config: Optional("--dataset"), value: ValueType::Path },
    ArgConfig { config: Optional("--min-pair-freq"), value: ValueType::Integer },
];

static ARGUMENTS: &[&[ArgConfig]] = &[
    PREPROCESSING_ARGUMENTS,
];

type ArgumentBookKeeping<'a> = HashMap<&'a str, (ValueType, bool)>;

impl Field {
    fn is_required(&self) -> bool {
        if let Required(_) = &self { true } else { false }
    }

    fn get_str(&self) -> &str {
        match &self { Required(s) | Optional(s) => s }
    }
}

impl ValueType {
    fn as_str(&self) -> &'static str {
        match self {
            ValueType::None => "",
            ValueType::Integer => "integer",
            ValueType::Path => "path",
        }
    }
}

fn collect_commands() -> Vec<(String, Vec<(Field, ValueType)>)> {
    COMMANDS.iter()
        .filter(|arg| arg.indir.is_some())
        .map(|argm| {
            let (cmd_name, indir) = (&argm.command, argm.indir.unwrap());
            let args: Vec<(Field, ValueType)> = ARGUMENTS[indir].iter()
                .map(|arg| (arg.config, arg.value)).collect();
            (cmd_name.to_string(), args)
        }).collect()
}

fn format_arguments(args: &[(Field, ValueType)]) -> String {
    args.iter()
        .map(|(a, val)| {
            let a_name = match a {
                Optional(s) | Required(s) => s,
            };
            let fmt_val_names = if *val == ValueType::None { String::new() } else { format!(" <{}>", val.as_str()) };
            match a {
                Optional(_) => format!("[{}{}]", a_name,  fmt_val_names),
                Required(_) => format!("{}{}", a_name, fmt_val_names),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_command(command: (String, Vec<(Field, ValueType)>)) -> String {
    let (cmd, cfgs) = command;
    if cfgs.is_empty() {
        return format!("{}", cmd);
    }
    format!("{}\t{}", cmd, format_arguments(&cfgs))
}

fn format_commands<'a>(commands: &'a [(String, Vec<(Field, ValueType)>)]) -> String {
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
    let res = args.iter()
        .map(|s|
            if is_first_char_alpha(s) {
                format!("\"{}\"", s)
            } else {
                s.clone()
            }
        )
        .collect::<Vec<_>>()
        .join(" ");
    println!("{:?}", res);
    return res;
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

fn parse_value(val_type: ValueType, argument: &str, val_arg: &str) -> Result<Value, String> {
    let val_str_rep = val_type.as_str();
    let fmt_err_usage = format!(
        "Could not parse <{}> from `{}`.\nArgument `{}` expected <{}>, got `{}`.",
        val_str_rep, val_arg, argument, val_str_rep, val_arg
    );
    match val_type {
        ValueType::Integer => match val_arg.parse::<i64>() {
            Ok(i) => Ok(Value::Integer(i)),
            Err(_) => Err(fmt_err_usage),
        },
        ValueType::Path => match val_arg.parse::<PathBuf>() {
            Ok(p) => Ok(Value::Path(p)),
            Err(_) => Err(fmt_err_usage),
        }
        _ => Ok(Value::None()),
    }
}

fn handle_command_values(
    args: &[String],
    req: &mut ArgumentBookKeeping,
    opt: &mut ArgumentBookKeeping,
    config: &mut Config,
    fmt_usage: &str,
) -> Result<(), String> {
    let mut expect_val = ValueType::None;
    let mut expect_val_for: &str = "";
    for (i, arg) in args.iter().skip(1).enumerate() {
        if expect_val != ValueType::None {
            let val = parse_value(expect_val, expect_val_for, arg);
            if let Err(err) = val {
                return Err(format!(
                    "{}\n{}\n{} -- parse error",
                    err, join_args(args),
                    " ".repeat(args_space_separated_len(args, i + 1)) + &repeated_underline(arg)
                ));
            }
            // modify_config(config, );
            expect_val = ValueType::None;
            expect_val_for = "";
            continue;
        }
        if let Some(val) = req.get_mut(arg.as_str()) {
            if val.1 { 
                return Err(repeated_config_err(args, i + 1));
            } else {
                expect_val = val.0;
                expect_val_for = arg;
                val.1 = true;
            }
        } else if let Some(val) = opt.get_mut(arg.as_str()) {
            if val.1 {
                return Err(repeated_config_err(args, i));
            } else {
                expect_val = val.0;
                expect_val_for = arg;
                val.1 = true;
            }    
        } else {
            return Err(format!(
                "unknown argument `{}`\nExpected usage:\n{}",
                arg, fmt_usage
            ));
        }
    }
    Ok(())
}

fn handle_command(command: &Argument, args: &[String], config: &mut Config) -> Result<(), String> {
    if command.indir.is_none() && args.len() > 1 {
        let args_entered = join_args(args);
        let cmd_len = args[0].len() + 1;
        return Err(
            format!("Command `{}` takes 0 arguments, got {}.\n\t{}\n\t{} -- unexpected arguments provided",
            command.command, args.len() - 1, args_entered, " ".repeat(cmd_len) + &"~".repeat(args_entered.len() - cmd_len)
        ));
    }
    if command.indir.is_some() {
        let subargs = ARGUMENTS[command.indir.unwrap()];
        let mut required_subargs: ArgumentBookKeeping = subargs
            .iter()
            .filter(|&a| a.config.is_required())
            .map(|arg| (arg.config.get_str(), (arg.value, false)))
            .collect();
        let mut optional_subargs: ArgumentBookKeeping = subargs
            .iter()
            .filter(|&a| !a.config.is_required())
            .map(|arg| (arg.config.get_str(), (arg.value, false)))
            .collect();
        let arg_configs: Vec<(Field<&str>, ValueType)> = [
            required_subargs.iter()
                .map(|(&a, (v, _))| (Field::Required(a), *v))
                .collect::<Vec<_>>(),
            optional_subargs.iter()
                .map(|(&a, (v, _))| (Field::Optional(a), *v))
                .collect::<Vec<_>>(),
        ].concat();
        let fmt_usage = format_command((String::from(command.command), arg_configs));
        if args.len() != 1 {
            if let Err(err) = handle_command_values(&args, &mut required_subargs, &mut optional_subargs, config, &fmt_usage) {
                return Err(err);
            }
        }
        else if !required_subargs.is_empty() {
            return Err(
                format!("Command `{}` expected at least {} commands, got 0.\n\nCommand usage:\n{}",
                command.command, required_subargs.len(), fmt_usage)
            );
        }
    }
    Ok(())
}

pub fn handle_args(args: &[String], def_config: &Config) -> Result<Config, String> {
    if args.is_empty() {
        return Err(usage_hint());
    }
    let mut config = def_config.clone();
    let command = &args[0];
    let cmd_idx = COMMANDS.iter().position(|&x| x.command == command);
    if cmd_idx.is_none() {
        return Err(format!("No command '{}' exists.\n\nAvailable commands:\n\t{}", command, command_hint()));
    }
    let cmd = &COMMANDS[cmd_idx.unwrap()];
    if let Err(error) = handle_command(cmd, &args, &mut config) {
        return Err(error);
    }
    Ok(config)
}