use crate::config::Config;
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Copy, Debug)]
enum Field<'a> {
    Optional(&'a str),
    Required(&'a str),
}
use Field::{Optional, Required};

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

#[derive(Clone, Copy, Debug)]
struct ArgConfig<'a> {
    config: Field<'a>,
    value: ValueType,
}

#[derive(Clone, Copy, Debug)]
struct CommandSpec<'a> {
    command: &'a str,
    indir: Option<usize>,
}

static COMMANDS: &[CommandSpec] = &[
    CommandSpec { command: "-pre-processing", indir: Some(0usize) },
];

static PREPROCESSING_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig { config: Required("--vocab-size"), value: ValueType::UInt },
    ArgConfig { config: Optional("--dataset"), value: ValueType::Path },
    ArgConfig { config: Optional("--min-pair-freq"), value: ValueType::UInt },
];

static ARGUMENTS: &[&[ArgConfig]] = &[
    PREPROCESSING_ARGUMENTS,
];

type ArgumentBookKeeping<'a> = HashMap<&'a str, (ValueType, bool)>;

impl Field<'_> {
    fn is_required(&self) -> bool {
        matches!(&self, Required(_))
    }

    fn get_str<'a>(&'a self) -> &'a str {
        match &self { Required(s) | Optional(s) => s }
    }
}

impl ValueType {
    fn as_str(&self) -> &'static str {
        match self {
            ValueType::None => "",
            ValueType::UInt => "unsigned integer",
            ValueType::Path => "path",
        }
    }
}

fn collect_commands<'a>() -> Vec<(String, Vec<(Field<'a>, ValueType)>)> {
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

fn parse_value(val_type: ValueType, argument: &str, val_arg: &str) -> Result<Value, String> {
    let val_str_rep = val_type.as_str();
    let fmt_err_usage = format!(
        "Could not parse <{}> from `{}`.\nArgument `{}` expected <{}>, got `{}`.",
        val_str_rep, val_arg, argument, val_str_rep, val_arg
    );
    match val_type {
        ValueType::UInt => match val_arg.parse::<u64>() {
            Ok(i) => Ok(Value::UInt(i)),
            Err(_) => Err(fmt_err_usage),
        },
        ValueType::Path => match val_arg.parse::<PathBuf>() {
            Ok(p) => Ok(Value::Path(p)),
            Err(_) => Err(fmt_err_usage),
        }
        _ => Ok(Value::None),
    }
}

fn missing_req_args(req: &ArgumentBookKeeping, command: &str, args: &str, fmt_usage: &str) -> Result<(), String> {
    let missing: Vec<(&str, ValueType)> = req.iter().filter(|(_, (_, b))| !*b).map(|(arg, (val, _))| (*arg, *val)).collect();
    if !missing.is_empty() {
        return Err(format!(
            "Missing required arguments for command `{}`:\n\t{}\n{}\n{}\n{} out of {} required fields missing.\n\nCommand usage:\n\t{}",
            command,
            missing.iter().map(|(arg, val)| 
                if *val != ValueType::None { format!("{} <{}>", arg, val.as_str()) }
                else { format!("{}", arg) }
            ).collect::<Vec<_>>().join("\n\t"),
            args, "~".repeat(args.len()), missing.len(), req.len(), fmt_usage
        ));
    }
    Ok(())
}

fn handle_command_values(
    args: &[String],
    req: &mut ArgumentBookKeeping,
    opt: &mut ArgumentBookKeeping,
    _config: &mut Config,
    fmt_usage: &str,
) -> Result<(), String> {
    let full_args = join_args(args);
    let (mut expect_val, mut expect_val_for) = (ValueType::None, "");
    for (i, arg) in args.iter().enumerate().skip(1) {
        if expect_val != ValueType::None {
            let val = parse_value(expect_val, expect_val_for, arg);
            if let Err(err) = val {
                return Err(format!(
                    "{}\n{}\n{} -- parse error", err, full_args, " ".repeat(args_space_separated_len(args, i)) + &repeated_underline(arg)
                ));
            }
            (expect_val, expect_val_for) = (ValueType::None, "");
            // TODO: update_config
            continue;
        }
        let mut appeared_req = false;
        if let Some(val) = req.get_mut(arg.as_str()) {
            if val.1 { 
                appeared_req = true;
            } else {
                (expect_val, expect_val_for, val.1) = (val.0, arg, true);
                continue;
            }
        }
        if let Some(val) = opt.get_mut(arg.as_str()) {
            if val.1 {
                return Err(repeated_config_err(args, i));
            } else {
                (expect_val, expect_val_for, val.1) = (val.0, arg, true);
                continue;
            }    
        }
        if appeared_req {
            return Err(repeated_config_err(args, i));
        }
        return Err(format!("unknown argument `{}`\nExpected usage:\n{}", arg, fmt_usage));
    }
    if expect_val != ValueType::None {
        return Err(format!(
            "In command `{}`: field `{}` expected `<{}>`, got nil.\n{}\n{}",
            args[0], expect_val_for, expect_val.as_str(), full_args, " ".repeat(full_args.len() - 1) + "^~~ -- missing value"
        ));
    }
    missing_req_args(req, &args[0], &full_args, fmt_usage)?;
    Ok(())
}

fn map_entry_check(map: &ArgumentBookKeeping, look_map: Option<&ArgumentBookKeeping>, key: &str, command: &str) -> Result<(), String> {
    let exists = map.contains_key(&key) || look_map.is_some_and(|m| m.contains_key(&key));
    if exists {
        return Err(format!(
            "Program Error: argument `{}` already exists in arg list for command `{}`.\nNot a problem with the args passed.\nError in argument collection phase of validation.",
            key, command
        ));
    }
    Ok(())
}

fn handle_command(command: &CommandSpec, args: &[String], config: &mut Config) -> Result<(), String> {
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

        let mut required_subargs: ArgumentBookKeeping = HashMap::new();
        let mut optional_subargs: ArgumentBookKeeping = HashMap::new();
        
        for arg in subargs.iter().filter(|&a| a.config.is_required()) {
            map_entry_check(&required_subargs, None, arg.config.get_str(), command.command)?;
            required_subargs.insert(arg.config.get_str(), (arg.value, false));
        }
        for arg in subargs.iter().filter(|&a| !a.config.is_required()) {
            map_entry_check(&optional_subargs, Some(&required_subargs), arg.config.get_str(), command.command)?;
            optional_subargs.insert(arg.config.get_str(), (arg.value, false));
        }
        
        let arg_configs: Vec<(Field, ValueType)> = subargs.iter().map(|&a| (a.config, a.value)).collect();
        let fmt_usage = format_command((String::from(command.command), arg_configs));
        handle_command_values(args, &mut required_subargs, &mut optional_subargs, config, &fmt_usage)?;
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
    if let Some(indir) = cmd.indir {
        if indir >= ARGUMENTS.len() {
            return Err(format!(
                "Program Error: command `{}` already holds index `{}` which is out of bounds for argument entries: >= {}.\nNot a problem with the args passed.\nError in argument collection phase of validation.",
                command, indir, ARGUMENTS.len()
            ));
        }
    }

    let cmd = &COMMANDS[cmd_idx];
    handle_command(cmd, &args, &mut config)?;
    Ok(config)
}