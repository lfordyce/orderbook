use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use csv::Trim;
use either::Either;
use erased_serde::serialize_trait_object;
use tap::Pipe;

use crate::cli::{Config, InputType};
use crate::core::{Engine, EngineError, OrderRequest, Side};

mod cli;
mod core;

pub trait LogTrait: erased_serde::Serialize + Send + Sync {
    fn get_label(&self) -> &String;
}

serialize_trait_object!(LogTrait);

#[derive(serde::Serialize)]
pub struct Row {
    pub label: String,
    pub values: Vec<u64>,
}

impl LogTrait for Row {
    fn get_label(&self) -> &String {
        &self.label
    }
}

#[derive(serde::Serialize)]
pub struct Acknowledgment {
    pub label: String,
    pub user_id: u64,
    pub user_order_id: u64,
}

impl LogTrait for Acknowledgment {
    fn get_label(&self) -> &String {
        &self.label
    }
}

#[derive(serde::Serialize)]
pub struct BookTop {
    pub label: String,
    pub side: String,
    pub price: u64,
    pub total_qty: u64,
}

impl LogTrait for BookTop {
    fn get_label(&self) -> &String {
        &self.label
    }
}

#[derive(Debug, thiserror::Error)]
enum ProcessingError {
    #[error(transparent)]
    EngineError(#[from] EngineError),
    #[error(transparent)]
    DispatchError(#[from] std::sync::mpsc::SendError<OrderRequest>),
    #[error(transparent)]
    Io(#[from] io::Error),
}

struct InputProcessor {
    rx: std::sync::mpsc::Receiver<OrderRequest>,
}

impl From<InputType> for InputProcessor {
    fn from(value: InputType) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || -> Result<(), ProcessingError> {
            let mut rdr = match value {
                InputType::File(path) => Either::Left(std::fs::File::open(path)?),
                InputType::Stdin => Either::Right(io::stdin()),
            }
            .pipe(|r| {
                csv::ReaderBuilder::new()
                    .trim(Trim::All)
                    .flexible(true)
                    .comment(Some(b'#'))
                    .has_headers(false)
                    .from_reader(r)
            });

            for record in rdr.records().flatten() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                match &record[0] {
                    "N" => {
                        tx.send(OrderRequest::Create {
                            user_id: record[1].parse::<u64>().unwrap(),
                            symbol: record[2].parse().unwrap(),
                            price: record[3].parse::<u64>().unwrap(),
                            qty: record[4].parse::<u64>().unwrap(),
                            side: record[5].parse::<Side>().unwrap(),
                            user_order_id: record[6].parse::<u64>().unwrap(),
                            unix_nano: now,
                        })?;
                    }
                    "C" => {
                        tx.send(OrderRequest::Cancel {
                            user_id: record[1].parse::<u64>().unwrap(),
                            user_order_id: record[2].parse::<u64>().unwrap(),
                            unix_nano: now,
                        })?;
                    }
                    "F" => {
                        tx.send(OrderRequest::FlushBook)?;
                    }
                    _ => {
                        // Skip unknown order transaction
                    }
                }
            }

            Ok(())
        });

        Self { rx }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let mut config = Config::parse();

    let (log_tx, log_rx) = std::sync::mpsc::channel::<Box<dyn LogTrait>>();

    let processor = InputProcessor::from(config.input.take().unwrap_or_default());
    std::thread::spawn(move || -> Result<(), ProcessingError> {
        let mut engine = Engine::new(log_tx);
        while let Ok(order) = processor.rx.recv() {
            engine.process(order)?;
        }
        Ok(())
    });

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_writer(io::stdout());
    while let Ok(record) = log_rx.recv() {
        csv_writer.serialize(record)?;
    }
    csv_writer.flush()?;
    Ok(())
}
