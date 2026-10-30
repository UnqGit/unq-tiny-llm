use crate::config::Config;

#[derive(Clone, Copy)]
enum Field {
    Optional(&'static str),
    Required(&'static str),
}

#[derive(Clone, Copy)]
struct ArgConfig {
    config: Field,
    value: Option<&'static [Field]>,
}

#[derive(Clone, Copy)]
struct Argument {
    command: &'static str,
    indir: Option<usize>,
}

static COMMANDS: &[Argument] = &[
    Argument { command: "-pre-processing", indir: Some(0usize) },
];

static PREPROCESSING_ARGUMENTS: &[ArgConfig] = &[
    ArgConfig { config: Field::Optional("--vocab-size"), value: Some(&[Field::Required("integer")]) },
    ArgConfig { config: Field::Optional("--dataset"), value: Some(&[Field::Required("path")]) },
    ArgConfig { config: Field::Optional("--min-pair-freq"), value: Some(&[Field::Required("integer")]) },
];

static ARGUMENTS: &[&[ArgConfig]] = &[
    PREPROCESSING_ARGUMENTS,
];

fn collect_commands() -> Vec<(String, Vec<(Field, Vec<Field>)>)> {
    COMMANDS.iter()
        .filter(|arg| arg.indir.is_some())
        .map(|argm| {
            let (cmd_name, indir) = (&argm.command, argm.indir.unwrap());
            let args: Vec<(Field, Vec<Field>)> = ARGUMENTS[indir].iter()
                .map(|arg| {
                    let cfg = arg.config;
                    let values: Vec<Field> = arg.value.unwrap_or(&[]).iter().map(|v| *v).collect();
                    (cfg, values)
                }).collect();
            (cmd_name.to_string(), args)
        }).collect()
}

fn format_arguments(args: &[(Field, Vec<Field>)]) -> String {
    args.iter()
        .map(|(a, vals)| {
            let a_name = match a {
                Field::Optional(s) | Field::Required(s) => s,
            };
            let val_names = vals
                .iter()
                .map(|&v| match v {
                    Field::Optional(s) => format!("[<{}>]", s),
                    Field::Required(s) => format!("<{}>", s),
                })
                .collect::<Vec<_>>()
                .join(" ");
            let fmt_val_names = if val_names.is_empty() { String::new() } else { format!(" {}", val_names) };
            let arg_section = match a {
                Field::Optional(_) => format!("[{}{}]", a_name,  fmt_val_names),
                Field::Required(_) => format!("{}{}", a_name, fmt_val_names),
            };
            arg_section
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_commands(commands: &[(String, Vec<(Field, Vec<Field>)>)]) -> String {
    commands.iter()
        .map(|(cmd, cfgs)| {
            if cfgs.is_empty() {
                return format!("{}", cmd);
            }
            format!("{}\t{}", cmd, format_arguments(cfgs))
        }).collect::<Vec<_>>().join("\n")
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

fn handle_command(command: &Argument, args: &[String], config: &mut Config) -> Result<(), String> {
    

    Ok(())
}

pub fn handle_args(args: &[String], config: &mut Config) -> Result<(), String> {
    if args.is_empty() {
        return Err(usage_hint());
    }
    let command = &args[0];
    let cmd_idx = COMMANDS.iter().position(|&x| x.command == command);
    if cmd_idx.is_none() {
        return Err(format!("No command '{}' exists.\nAvailable commands:\n\t{}", command, command_hint()));
    }
    let cmd = &COMMANDS[cmd_idx.unwrap()];
    if let Err(error) = handle_command(cmd, &args, config) {
        return Err(error);
    }
    Ok(())
}