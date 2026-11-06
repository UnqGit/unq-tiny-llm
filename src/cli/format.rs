use crate::cli::types::ArgConfig;

pub fn format_arguments<'a>(args: &[ArgConfig<'a>]) -> String {
    args.iter()
        .map(|arg| {
            let a_name = arg.config;
            let fmt_val_names = if let Some(val) = arg.value { format!(" <{}>", val.as_str()) } else { String::new() };
            match arg.required {
                false => format!("[{}{}]", a_name,  fmt_val_names),
                true => format!("{}{}", a_name, fmt_val_names),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_command(command: (String, Vec<ArgConfig>)) -> String {
    let (cmd, cfgs) = command;
    if cfgs.is_empty() {
        return cmd;
    }
    format!("{}\t{}", cmd, format_arguments(&cfgs))
}

pub fn format_commands<'a>(commands: &'a [(String, Vec<ArgConfig>)]) -> String {
    commands.iter().map(|f| format_command(f.clone())).collect::<Vec<_>>().join("\n\t")
}

pub fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|s| s.clone())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn args_space_separated_len(args: &[String], i: usize) -> usize {
    args[0..i]
        .iter()
        .fold(0, |l, s| l + s.len() + 1)
}

pub fn repeated_underline(arg: &str) -> String {
    "~".repeat(arg.len())
}