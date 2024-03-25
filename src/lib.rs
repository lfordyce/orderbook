use erased_serde::serialize_trait_object;
use std::io;

pub type Error = Box<dyn std::error::Error + Sync + Send>;
fn process_order_file<T: io::Read>(mut reader: csv::Reader<T>) -> Result<(), Error> {
    Ok(())
}

pub trait LogTrait: erased_serde::Serialize + Send +Sync {
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
    pub values: Vec<u64>,
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
    pub values: Vec<u64>,
}

impl LogTrait for BookTop {
    fn get_label(&self) -> &String {
        &self.label
    }
}
