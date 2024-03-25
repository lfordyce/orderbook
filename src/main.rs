use crate::core::{Engine, Side};
use orderbook::LogTrait;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{error::Error, io};

mod core;

fn main() {
    let mut child = vec![];

    let (tx, rx) = std::sync::mpsc::channel();
    let (log_tx, log_rx) = std::sync::mpsc::channel::<Box<dyn LogTrait>>();

    child.push(std::thread::spawn(move || {
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .comment(Some(b'#'))
            .has_headers(false)
            .from_reader(io::stdin());
        for result in rdr.records() {
            if let Ok(record) = result {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                match &record[0] {
                    "N" => {
                        tx.send(core::OrderRequest::Create {
                            user_id: record[1].trim().parse::<u64>().unwrap(),
                            symbol: record[2].trim().parse().unwrap(),
                            price: record[3].trim().parse::<u64>().unwrap(),
                            qty: record[4].trim().parse::<u64>().unwrap(),
                            side: record[5].trim().parse::<Side>().unwrap(),
                            user_order_id: record[6].trim().parse::<u64>().unwrap(),
                            unix_nano: now,
                        })
                        .unwrap_or_else(|e| eprintln!("{}", e));
                    }
                    "C" => {
                        tx.send(core::OrderRequest::Cancel {
                            user_id: record[1].trim().parse::<u64>().unwrap(),
                            user_order_id: record[2].trim().parse::<u64>().unwrap(),
                            unix_nano: now,
                        })
                        .unwrap_or_else(|e| eprintln!("{}", e));
                    }
                    "F" => {
                        tx.send(core::OrderRequest::FlushBook)
                            .unwrap_or_else(|e| eprintln!("{}", e));
                    }
                    _ => continue,
                }
            }
        }
        drop(tx);
    }));

    child.push(std::thread::spawn(move || {
        let mut engine = Engine::new("", log_tx);
        while let Ok(order) = rx.recv() {
            if let Err(err) = engine.process(order) {
                eprintln!("something went wrong: {}", err);
            };
            // println!("{:?}", order);
        }
    }));

    for c in child {
        c.join().expect("TODO: panic message");
    }

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(io::stdout());
    while let Ok(record) = log_rx.recv() {
        csv_writer.serialize(record).unwrap();
        // csv_writer.write_record(&record).unwrap();
    }
    csv_writer.flush().unwrap()
}
