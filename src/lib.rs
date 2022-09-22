pub mod command;
mod entry;
pub mod errors;
mod file;

use entry::Entry;
use errors::{KvsError, Result};
use file::DBFile;
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

const SET: u16 = 0;
const DEL: u16 = 1;
const Max: u64 = 1000;

#[macro_use]
extern crate failure_derive;
extern crate core;

#[derive(Debug)]
pub struct KvStore {
    pub data: HashMap<String, u64>,
    pub db_file: DBFile,
    pub dir_path: PathBuf,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        // get the file path
        let mut path = path.into();
        // get the dir path
        let dir_path = path.clone();
        path.push("kvs");
        path.set_extension("txt");

        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;

        let offset = file.metadata()?.size();
        let db_file = DBFile { file, offset };

        let mut kv_store = KvStore {
            data: HashMap::new(),
            db_file,
            dir_path,
        };

        kv_store.restructure()?;

        Ok(kv_store)
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // 1. create an new entry
        let entry = Entry::new(SET, key.clone(), value.clone());

        // 2. get the offset
        let offset = self.db_file.offset;

        // 3. flush in entry in end of the file
        self.db_file.write(entry)?;

        // 4. update the data map
        self.data.insert(key, offset);

        if self.db_file.file.metadata().unwrap().len() >= Max {
            self.compaction()?;
        }

        // return
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // 1. get the data index from the file
        self.restructure()?;

        // 2. check the key exit?
        if !self.data.contains_key(key.as_str()) {
            return Ok(None);
        }

        // 3. get the entry by offset
        let offset = self.data.get(key.as_str()).unwrap();
        // if the offset > file.offset , return FileEnd error
        let entry = self.db_file.read(*offset)?;

        // 4. get the value from entry
        Ok(Some(entry.value))
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        // 2. check the key exit?
        if !self.data.contains_key(key.as_str()) {
            return Err(KvsError::KeyNotExit);
        }

        // 3. create an 'rm' entry
        let entry = Entry::new(DEL, key, String::new());

        // 4. push the entry in the log file
        self.db_file.write(entry)?;

        Ok(())
    }

    // may be bug
    pub fn restructure(&mut self) -> Result<()> {
        let mut offset = 0;

        loop {
            if offset >= self.db_file.offset {
                break;
            }
            // 1. get the entry
            let entry = match self.db_file.read(offset) {
                Ok(e) => e,
                Err(err) => match err {
                    KvsError::FileEnd => break,
                    _ => return Err(err),
                },
            };

            // check the mark
            if entry.mark == DEL {
                // check the key exit in the data
                if self.data.contains_key(entry.key.as_str()) {
                    self.data.remove(entry.key.as_str());
                }
                offset += entry.size() as u64;
                continue;
            }

            // 2. update the data index
            self.data.insert(entry.key.clone(), offset as u64);

            // 4. update the offset
            offset += entry.size() as u64;
        }

        Ok(())
    }

    /// only used in data exited
    pub fn compaction(&mut self) -> Result<()> {
        let mut offset = 0;
        let mut vaild_entry = Vec::new();
        loop {
            if offset >= self.db_file.offset {
                break;
            }

            let entry = self.db_file.read(offset)?;

            if self.data.contains_key(entry.key.as_str()) {
                let off = self.data.get(entry.key.as_str()).unwrap();
                if *off == offset {
                    vaild_entry.push(entry.clone());
                }
            }

            offset += entry.size() as u64;
        }

        if vaild_entry.len() > 0 {
            let mut tmp_path = self.dir_path.clone();
            tmp_path.push("tmp_kvs");
            tmp_path.set_extension("txt");
            let new_file = File::options()
                .write(true)
                .read(true)
                .create(true)
                .append(true)
                .open(tmp_path.clone())?;

            let mut new_db_file = DBFile {
                file: new_file,
                offset: 0,
            };

            for i in vaild_entry {
                let write_offset = new_db_file.offset;
                new_db_file.write(i.clone())?;
                // update the data
                self.data.insert(i.key, write_offset);
            }

            // get the olf file path
            let mut old_path = self.dir_path.clone();
            old_path.push("kvs");
            old_path.set_extension("txt");
            // remove the old file
            std::fs::remove_file(old_path.clone())?;
            // update the old file name
            std::fs::rename(tmp_path, old_path)?;

            self.db_file = new_db_file;
        }
        Ok(())
    }
}
