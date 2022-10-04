use crate::entry::{Entry, HEADERSIZE};
use crate::errors::Result;
use crate::KvsError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::os::unix::fs::FileExt;
use std::path::PathBuf;

pub trait FileIo {
    fn read(&self, offset: u64) -> Result<Entry>;
    fn write(&mut self, entry: Entry) -> Result<()>;
}

#[derive(Debug)]
pub struct DBFile {
    pub active_file: File,
    pub file_path: PathBuf,
    pub offset: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataValue {
    pub file_path: PathBuf,
    pub file_offset: u64,
}

#[derive(Debug, Clone)]
// immutable文件结构
pub struct ImmutableFile {
    pub data_path: PathBuf,
    pub data_offset: u64,
}

impl FileIo for DBFile {
    fn read(&self, offset: u64) -> Result<Entry> {
        // if offset >= the file offset, that mean at the end of file now
        // self.file.metadata().unwrap().len()
        if offset >= self.offset {
            return Err(KvsError::FileEnd);
        }

        // 1. create an buf
        let mut buf = [0; HEADERSIZE as usize];

        // 2. get the header
        let header_size = self.active_file.read_at(buf.as_mut_slice(), offset)?;

        // 3. get the entry
        let mut entry = Entry::decode(buf.to_vec());

        let mut offset = offset + header_size as u64;

        // 5. get the key
        let mut buf = vec![0; entry.key_size as usize];
        let read_key_size = self.active_file.read_at(buf.as_mut_slice(), offset)?;
        entry.key = String::from_utf8(buf)?;
        offset += read_key_size as u64;

        // 6. get the value
        let mut buf = vec![0; entry.value_size as usize];
        self.active_file.read_at(&mut buf, offset)?;
        entry.value = String::from_utf8(buf)?;

        Ok(entry)
    }

    fn write(&mut self, entry: Entry) -> Result<()> {
        let encode = Entry::encode(entry.clone())?;
        self.active_file.write_at(encode.as_slice(), self.offset)?;
        self.active_file.flush()?;
        self.offset += entry.size() as u64;
        self.active_file.flush()?;
        Ok(())
    }
}

impl FileIo for ImmutableFile {
    fn read(&self, offset: u64) -> Result<Entry> {
        let mut buf = [0; HEADERSIZE as usize];
        let file = File::open(self.data_path.clone())?;

        let mut size = file.read_at(buf.as_mut_slice(), offset)?;

        // get the entry
        let mut entry = Entry::decode(buf.to_vec());
        let mut offset = offset + size as u64;

        // get the key
        let mut buf = vec![0; entry.key_size as usize];
        size = file.read_at(buf.as_mut_slice(), offset)?;
        entry.key = String::from_utf8(buf)?;
        offset += size as u64;

        // get the value
        let mut buf = vec![0; entry.value_size as usize];
        size = file.read_at(buf.as_mut_slice(), offset)?;
        entry.value = String::from_utf8(buf)?;

        Ok(entry)
    }

    fn write(&mut self, entry: Entry) -> Result<()> {
        todo!()
    }
}

impl FileIo for DataValue {
    fn read(&self, offset: u64) -> Result<Entry> {
        let mut buf = [0; HEADERSIZE as usize];
        let file = File::open(self.file_path.clone())?;

        let mut size = file.read_at(buf.as_mut_slice(), offset)?;

        // get the entry
        let mut entry = Entry::decode(buf.to_vec());
        let mut offset = offset + size as u64;

        // get the key
        let mut buf = vec![0; entry.key_size as usize];
        size = file.read_at(buf.as_mut_slice(), offset)?;
        entry.key = String::from_utf8(buf)?;
        offset += size as u64;

        // get the value
        let mut buf = vec![0; entry.value_size as usize];
        size = file.read_at(buf.as_mut_slice(), offset)?;
        entry.value = String::from_utf8(buf)?;

        Ok(entry)
    }

    fn write(&mut self, entry: Entry) -> Result<()> {
        todo!()
    }
}
