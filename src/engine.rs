use crate::errors::*;
use std::path::Path;

pub enum EngineType {
    KvsStore,
    SledKvsEngine,
}

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<()>;
    fn remove(&mut self, key: String) -> Result<()>;
    fn open(&mut self, dir: &Path) -> Result<()>;
}
