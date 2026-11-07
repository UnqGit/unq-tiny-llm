use crate::cli::types::{ArgValue, Value};

pub fn get_some_argval(arg: &ArgValue) -> Result<Value, String> {
    match &arg.value {
        Some(value) => Ok(value.clone()),
        _ => Err(format!("Expected a value in argument `ArgValue`, got None."))
    }
}

pub fn unexpected_type_err(config_name: &str, expected_type: &str) -> String {
    format!("Program error: unexpected value received for `{}`, expcted `<{}>`", config_name, expected_type)
}