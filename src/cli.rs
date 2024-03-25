use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Clone, Debug)]
pub struct Config {
    #[arg(short, long, value_name = "ORDER FILE SOURCE")]
    pub input: Option<InputType>,
}

#[derive(Debug, Default, Clone)]
pub enum InputType {
    #[default]
    Stdin,
    File(PathBuf),
}

impl From<&str> for InputType {
    fn from(s: &str) -> Self {
        InputType::File(s.to_owned().into())
    }
}
