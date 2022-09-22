use crate::entry::{Entry, HEADERSIZE};
use crate::errors::Result;
use crate::KvsError;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::FileExt;

#[derive(Debug)]
pub struct DBFile {
    pub file: File,
    pub offset: u64,
}

impl DBFile {
    /// input: offset: u64
    /// return Ok(None) when the file end
    /// return Err() when read error
    pub fn read(&mut self, offset: u64) -> Result<Entry> {
        // if offset >= the file offset, that mean at the end of file now
        // self.file.metadata().unwrap().len()
        if offset >= self.offset {
            return Err(KvsError::FileEnd);
        }

        // 1. create an buf
        let mut buf = [0; HEADERSIZE as usize];

        // 2. get the header
        let header_size = self.file.read_at(buf.as_mut_slice(), offset)?;

        // 3. get the entry
        let mut entry = Entry::decode(buf.to_vec());

        let mut offset = offset + header_size as u64;

        // 5. get the key
        let mut buf = vec![0; entry.key_size as usize];
        let read_key_size = self.file.read_at(buf.as_mut_slice(), offset)?;
        entry.key = String::from_utf8(buf)?;
        offset += read_key_size as u64;

        // 6. get the value
        let mut buf = vec![0; entry.value_size as usize];
        self.file.read_at(&mut buf, offset)?;
        entry.value = String::from_utf8(buf)?;

        Ok(entry)
    }

    pub fn write(&mut self, entry: Entry) -> Result<()> {
        let encode = Entry::encode(entry.clone())?;
        self.file.write_at(encode.as_slice(), self.offset)?;

        self.offset += entry.size() as u64;
        self.file.flush()?;
        Ok(())
    }
}
