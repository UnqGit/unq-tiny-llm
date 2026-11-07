use crate::cli::types::{ArgValue, Value};

pub fn get_some_argval(arg: &ArgValue) -> Result<Value, String> {
    match &arg.value {
        Some(value) => Ok(value.clone()),
        _ => Err("Expected a value in argument `ArgValue`, got None.".to_string()),
    }
}

pub fn unexpected_type_err(config_name: &str, expected_type: &str) -> String {
    format!(
        "Program error: unexpected value received for `{config_name}`, expcted `<{expected_type}>`"
    )
}
