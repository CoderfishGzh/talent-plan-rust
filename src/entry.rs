use crate::errors::{EntryError, KvsError, Result};
pub const HEADERSIZE: u32 = 10;

#[derive(PartialEq, Debug, Clone)]
pub struct Entry {
    pub mark: u16,
    pub key_size: u32,
    pub value_size: u32,
    pub key: String,
    pub value: String,
}

impl Entry {
    pub fn new(mark: u16, key: String, value: String) -> Self {
        let key_size = key.len() as u32;
        let value_size = value.len() as u32;
        Entry {
            mark,
            key_size,
            value_size,
            key,
            value,
        }
    }

    pub fn size(&self) -> u32 {
        self.key_size + self.value_size + HEADERSIZE
    }

    pub fn encode(entry: Entry) -> Result<Vec<u8>> {
        // 1. check the entry size
        if entry.size() <= HEADERSIZE {
            Err(KvsError::EntryError(EntryError::Illegal))?;
        }

        // 2. create the deserialize vec
        let mut ret: Vec<u8> = Vec::new();

        ret.extend(entry.mark.to_be_bytes());
        ret.extend(entry.key_size.to_be_bytes());
        ret.extend(entry.value_size.to_be_bytes());
        ret.extend(entry.key.as_bytes());
        ret.extend(entry.value.as_bytes());

        Ok(ret)
    }

    // get entry only contain header
    pub fn decode(buf: Vec<u8>) -> Self {
        let mark = u16::from_be_bytes(buf[0..2].try_into().unwrap());
        let key_size = u32::from_be_bytes(buf[2..6].try_into().unwrap());
        let value_size = u32::from_be_bytes(buf[6..10].try_into().unwrap());

        Self {
            mark,
            key_size,
            value_size,
            key: "".to_string(),
            value: "".to_string(),
        }
    }
}
