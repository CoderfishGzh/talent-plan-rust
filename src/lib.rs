use std::collections::HashMap;

#[derive(Debug)]
pub struct KvStore {
    pub data: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        match self.data.get(&key) {
            Some(_) => {
                self.data.insert(key, value);
            }
            None => {
                self.data.insert(key, value);
            }
        }
    }
    pub fn get(&self, key: String) -> Option<String> {
        match self.data.get(&key) {
            Some(v) => {
                return Some(v.clone());
            }
            None => {
                return None;
            }
        }
    }
    pub fn remove(&mut self, key: String) {
        self.data.remove(&key);
    }
}
