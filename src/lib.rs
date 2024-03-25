use csv::Trim;
use erased_serde::serialize_trait_object;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

mod core;
use crate::core::{Engine, EngineError, OrderRequest, Side};

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
}

struct InputProcessor<I> {
    tx: std::sync::mpsc::Sender<OrderRequest>,
    reader: csv::Reader<I>,
}

impl<I: io::Read> InputProcessor<I> {
    fn new(reader: csv::Reader<I>, tx: std::sync::mpsc::Sender<OrderRequest>) -> Self {
        InputProcessor { tx, reader }
    }

    fn run(&mut self) -> Result<(), ProcessingError> {
        for record in self.reader.records().flatten() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            match &record[0] {
                "N" => {
                    self.tx.send(OrderRequest::Create {
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
                    self.tx.send(OrderRequest::Cancel {
                        user_id: record[1].parse::<u64>().unwrap(),
                        user_order_id: record[2].parse::<u64>().unwrap(),
                        unix_nano: now,
                    })?;
                }
                "F" => {
                    self.tx.send(OrderRequest::FlushBook)?;
                }
                _ => {
                    // Skip unknown order transaction
                }
            }
        }
        Ok(())
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    let mut thread_handlers = vec![];

    let (tx, rx) = std::sync::mpsc::channel();
    let (log_tx, log_rx) = std::sync::mpsc::channel::<Box<dyn LogTrait>>();
    let rdr = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .comment(Some(b'#'))
        .has_headers(false)
        .from_reader(io::stdin());

    let mut input = InputProcessor::new(rdr, tx);

    thread_handlers.push(std::thread::spawn(
        move || -> Result<(), ProcessingError> {
            input.run()?;
            Ok(())
        },
    ));

    thread_handlers.push(std::thread::spawn(
        move || -> Result<(), ProcessingError> {
            let mut engine = Engine::new("", log_tx);
            while let Ok(order) = rx.recv() {
                // println!("{:?}", order);
                engine.process(order)?;
            }
            Ok(())
        },
    ));

    for handle in thread_handlers {
        handle.join().unwrap()?;
    }

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_writer(io::stdout());
    while let Ok(record) = log_rx.recv() {
        csv_writer
            .serialize(record)
            .unwrap_or_else(|e| eprintln!("failed to serialize CSV record {}", e));
    }
    csv_writer.flush()?;
    Ok(())
}
