use clap::Parser;
use orderbook::run;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser, Clone, Debug)]
struct Args {}

#[derive(Debug, Default)]
enum Input {
    #[default]
    Stdin,
    File(PathBuf),
}

impl From<&str> for Input {
    fn from(s: &str) -> Self {
        Input::File(s.to_owned().into())
    }
}

impl io::Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Input::Stdin => io::stdin().lock().read(buf),
            Input::File(path) => fs::File::open(path)?.read(buf),
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
