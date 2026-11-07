use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ValueType {
    Path,
    UInt,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    UInt(usize),
    Path(PathBuf),
}

#[derive(Clone, Copy, Debug)]
pub enum ParseState<'a> {
    ExpectingArgument,
    ExpectingValue {
        value_type: ValueType,
        argument: &'a str,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct ArgConfig<'a> {
    pub config: &'a str,
    pub required: bool,
    pub value: Option<ValueType>,
}

#[derive(Clone, Copy, Debug)]
pub struct CommandSpec<'a> {
    pub command: &'a str,
    pub arguments: Option<&'a [ArgConfig<'a>]>,
}

#[derive(Clone, Copy, Debug)]
pub struct ArgumentState {
    pub value_type: Option<ValueType>,
    pub visited: bool,
}

#[derive(Clone, Debug)]
pub struct ArgValue {
    pub argument: String,
    pub value: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct ParsedCommand {
    pub command: String,
    pub updated_args: Vec<ArgValue>,
}

pub type ArgumentRegistry<'a> = HashMap<&'a str, ArgumentState>;

impl ValueType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UInt => "unsigned integer",
            Self::Path => "path",
        }
    }
}
