pub mod command;
pub mod engine;
mod entry;
pub mod errors;
mod file;

use crate::errors::FileError;

use crate::file::{DataValue, FileIo, ImmutableFile};
use entry::Entry;
use errors::{KvsError, Result};
use file::DBFile;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use tempfile::tempfile;
use walkdir::{DirEntry, WalkDir};

const SET: u16 = 0;
const DEL: u16 = 1;
const MAX: u64 = 10000;

#[macro_use]
extern crate failure_derive;
extern crate core;

#[derive(Debug)]
pub struct KvStore {
    pub data: HashMap<String, DataValue>,
    // the file can push now
    pub active_file: DBFile,
    // record the immutable files
    pub immutable_files: Vec<ImmutableFile>,
    pub dir_path: PathBuf,
    pub hint_file: PathBuf,
    // 每过生成max_immytable个immutable 就会执行一次合并操作
    pub max_immytable: u64,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        // 1. check the dir path is vaild
        let dir_path = path.into();
        if !dir_path.clone().is_dir() {
            return Err(KvsError::File(FileError::PathIllegal));
        }

        // 2. get the file path from dir
        let mut data_index_path: PathBuf = PathBuf::new();
        let mut data_path: Vec<PathBuf> = Vec::new();
        for file in WalkDir::new(dir_path.clone())
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|e| illage_data_index(e) || illage_data_file(e))
        {
            let path = file?;
            // the data index file
            if illage_data_index(&path.clone()) {
                data_index_path = path.clone().into_path();
            }
            // the immutable file
            data_path.push(path.clone().into_path());
        }

        let mut store = KvStore {
            data: Default::default(),
            active_file: DBFile {
                active_file: tempfile()?,
                file_path: Default::default(),
                offset: 0,
            },
            immutable_files: vec![],
            dir_path,
            hint_file: data_index_path,
            max_immytable: 3,
        };

        // update the active file and the immutable files
        for p in data_path.into_iter() {
            let data_path = p;
            let imp = ImmutableFile {
                data_path: data_path.clone(),
                data_offset: data_path.metadata()?.size(),
            };
            store.immutable_files.push(imp);
        }

        if store.immutable_files.len() == 0 {
            // active 文件不存在，即第一次使用该store
            let mut path = store.dir_path.clone();
            path.push("0_kvs");
            path.set_extension("txt");

            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path.clone())?;

            let active_file = DBFile {
                active_file: file,
                file_path: path,
                offset: 0,
            };

            store.active_file = active_file;
        } else {
            // 使用immutable 数组 最后一个文件构造成DBFile
            let table = store.immutable_files.last().unwrap();
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(table.data_path.clone())?;

            store.active_file = DBFile {
                active_file: file,
                file_path: table.data_path.clone(),
                offset: table.data_offset,
            };
            store.immutable_files.pop();
        }

        // restruct the data index
        store.restructure()?;
        Ok(store)
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // create new entry
        let entry = Entry::new(SET, key.clone(), value);

        // get the offset
        let offset = self.active_file.offset;

        self.active_file.write(entry)?;

        // update the data map
        self.data.insert(
            key,
            DataValue {
                file_path: self.active_file.file_path.clone(),
                file_offset: offset,
            },
        );

        // check the file offset
        if self.active_file.offset >= MAX {
            self.updata_active_file()?;
        }

        if self.max_immytable == 0 {
            self.compaction()?;
            self.max_immytable = 3;
        }

        // return
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // check the key exit
        if !self.data.contains_key(key.as_str()) {
            return Ok(None);
        }

        // get the key
        let value = self.data.get(key.as_str()).unwrap();

        // get the entry
        let entry = value.read(value.file_offset)?;

        Ok(Some(entry.value))
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        // check the key exit
        if !self.data.contains_key(key.as_str()) {
            return Err(KvsError::KeyNotExit);
        }

        // create the DEL entry
        let entry = Entry::new(DEL, key, String::new());

        // push the entry into the active file
        self.active_file.write(entry.clone())?;

        // update the data
        self.data.remove(entry.key.as_str());

        if self.active_file.offset >= MAX {
            self.updata_active_file()?;
        }

        if self.max_immytable == 0 {
            self.compaction()?;
            self.max_immytable = 3;
        }

        Ok(())
    }

    pub fn restructure(&mut self) -> Result<()> {
        let path = self.hint_file.as_path();
        if path.exists() {
            let string = std::fs::read_to_string(path)?;
            let decode: HashMap<String, DataValue> = serde_json::from_str(string.as_str())?;
            self.data = decode;
            return Ok(());
        }

        // 如果不存在hint文件，需要从多个immutable中复现data index
        self.restructure_immutable()?;
        self.restructure_active_file()?;
        Ok(())
    }

    pub fn restructure_active_file(&mut self) -> Result<()> {
        let mut offset = 0;
        loop {
            if offset >= self.active_file.offset {
                break;
            }

            // get the entry
            let entry = self.active_file.read(offset)?;

            // update the data
            self.data.insert(
                entry.key.clone(),
                DataValue {
                    file_path: self.active_file.file_path.clone(),
                    file_offset: offset,
                },
            );

            // check id the entry is DEL
            if entry.mark == DEL {
                self.data.remove(entry.key.as_str());
            }

            // update the offset
            offset += entry.size() as u64;
        }

        Ok(())
    }

    pub fn restructure_immutable(&mut self) -> Result<()> {
        let mut data: HashMap<String, DataValue> = HashMap::new();

        for immutable in self.immutable_files.iter() {
            let mut offset = 0;
            let path = immutable.data_path.clone();
            loop {
                if offset >= immutable.data_offset {
                    break;
                }
                // 读取entry
                let entry = immutable.read(offset)?;
                // 插入或更新key的值
                data.insert(
                    entry.key.clone(),
                    DataValue {
                        file_path: path.clone(),
                        file_offset: offset,
                    },
                );
                // 如果是Mark 是 DEL 删除
                if entry.mark == DEL {
                    data.remove(entry.key.as_str());
                }
                // update the offset
                offset += entry.size() as u64;
            }
        }

        self.data = data;
        Ok(())
    }

    pub fn updata_active_file(&mut self) -> Result<()> {
        let immutable = ImmutableFile {
            data_path: self.active_file.file_path.clone(),
            data_offset: self.active_file.offset,
        };

        let mut active_file_path = self.dir_path.clone();
        let file_name = (self.immutable_files.len() + 2).to_string() + "_kvs";
        active_file_path.push(file_name);
        active_file_path.set_extension("txt");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(active_file_path.clone())?;

        let active_file = DBFile {
            active_file: file,
            file_path: active_file_path,
            offset: 0,
        };

        self.immutable_files.push(immutable);
        self.active_file = active_file;

        self.max_immytable -= 1;
        Ok(())
    }

    // 合并操作
    // 从immutable文件中拿出entry与当前的data里面的数据进行对比
    // 如果DataValue相同，则保存在vaild_entry里面
    pub fn compaction(&mut self) -> Result<()> {
        let mut vaild_entry: Vec<Entry> = Vec::new();
        for immutable in self.immutable_files.iter() {
            let mut offset = 0;
            loop {
                if offset >= immutable.data_offset {
                    break;
                }

                let entry = immutable.read(offset)?;
                // check the entry key exit in the data
                if self.data.contains_key(entry.key.as_str()) {
                    let data_value = self.data.get(entry.key.as_str()).unwrap();
                    if *data_value.file_path == immutable.data_path
                        && data_value.file_offset == offset
                    {
                        vaild_entry.push(entry.clone());
                    }
                }

                offset += entry.size() as u64;
            }
        }

        if !vaild_entry.is_empty() {
            // del all the immutable file
            self.clear_immutable_file()?;
            let mut imtable =
                new_tmp_file(self.dir_path.clone(), self.immutable_files.len() as u64)?;
            for i in vaild_entry {
                let av = DataValue {
                    file_path: imtable.data_path.clone(),
                    file_offset: imtable.data_offset.clone(),
                };
                imtable.write(i.clone())?;
                self.data.insert(i.key.clone(), av);
                if imtable.data_offset >= MAX {
                    self.immutable_files.push(imtable.clone());
                    // update the imtable
                    imtable =
                        new_tmp_file(self.dir_path.clone(), self.immutable_files.len() as u64)?;
                }
            }
        }

        self.max_immytable = 0;
        Ok(())
    }

    fn clear_immutable_file(&mut self) -> Result<()> {
        loop {
            let immutable = self.immutable_files.last();
            match immutable {
                None => {
                    break;
                }
                Some(table) => {
                    fs::remove_file(table.data_path.clone())?;
                }
            }
        }
        Ok(())
    }

    // pub fn compaction(&mut self) -> Result<()> {
    //     let mut offset = 0;
    //     let mut vaild_entry = Vec::new();
    //     loop {
    //         if offset >= self.db_file.offset {
    //             break;
    //         }
    //
    //         let entry = self.db_file.read(offset)?;
    //
    //         if self.data.contains_key(entry.key.as_str()) {
    //             let off = self.data.get(entry.key.as_str()).unwrap();
    //             if *off == offset {
    //                 vaild_entry.push(entry.clone());
    //             }
    //         }
    //
    //         offset += entry.size() as u64;
    //     }
    //
    //     if vaild_entry.len() > 0 {
    //         let mut tmp_path = self.dir_path.clone();
    //         tmp_path.push("tmp_kvs");
    //         tmp_path.set_extension("txt");
    //         let new_file = File::options()
    //             .write(true)
    //             .read(true)
    //             .create(true)
    //             .append(true)
    //             .open(tmp_path.clone())?;
    //
    //         let mut new_db_file = DBFile {
    //             file: new_file,
    //             offset: 0,
    //         };
    //
    //         for i in vaild_entry {
    //             let write_offset = new_db_file.offset;
    //             new_db_file.write(i.clone())?;
    //             // update the data
    //             self.data.insert(i.key, write_offset);
    //         }
    //
    //         // get the olf file path
    //         let mut old_path = self.dir_path.clone();
    //         old_path.push("kvs");
    //         old_path.set_extension("txt");
    //         // remove the old file
    //         std::fs::remove_file(old_path.clone())?;
    //         // update the old file name
    //         std::fs::rename(tmp_path, old_path)?;
    //
    //         self.db_file = new_db_file;
    //     }
    //     Ok(())
    // }
}

fn illage_data_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with("_kvs.txt"))
        .unwrap_or(false)
}

fn illage_data_index(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with("_index.txt"))
        .unwrap_or(false)
}

pub fn test() -> Result<()> {
    let mut store = KvStore::open("./")?;

    for i in 0..1000 {
        store.set(i.to_string(), i.to_string())?;
    }

    for i in 0..1000 {
        println!("value : {:?}", store.get(i.to_string()));
    }

    Ok(())
}

pub fn new_tmp_file(dir: PathBuf, num: u64) -> Result<ImmutableFile> {
    let mut path = dir.clone();
    let path_string = num.to_string() + "_kvs";
    path.push(path_string);
    path.set_extension("txt");

    Ok(ImmutableFile {
        data_path: path,
        data_offset: 0,
    })
}
