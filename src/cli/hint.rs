use crate::cli::{config::schema::COMMANDS, format::format_commands, types::ArgConfig};

fn collect_commands<'a>() -> Vec<(String, Vec<ArgConfig<'a>>)> {
    COMMANDS
        .iter()
        .map(|argm| {
            let cmd_name = argm.command.to_string();
            let cfgs = match argm.arguments {
                Some(a) => Vec::from(a),
                None => Vec::new(),
            };
            (cmd_name, cfgs)
        })
        .collect()
}

pub fn command_hint() -> String {
    let res = collect_commands();
    format_commands(&res)
}

pub fn usage_hint() -> String {
    let mut result = "Usage: cargo run -- <command> <arguments>".to_string();

    let commands: Vec<String> = COMMANDS.iter().map(|arg| arg.command.to_string()).collect();

    result += format!("\n\nAvailable commands:\n\t{}", commands.join("\n\t")).as_str();
    result += format!("\n\nCommand Usage:\n\t{}", command_hint()).as_str();

    result
}
