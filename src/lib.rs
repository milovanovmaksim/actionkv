use std::fs::OpenOptions;
use std::path::Path;
use std::{fs::File, collections::HashMap};
use std::io::{BufReader, SeekFrom, self, BufWriter};
use serde_derive::{Deserialize, Serialize};
use std::io::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;


pub type ByteString = Vec<u8>;

pub type ByteStr = [u8];


#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}


#[derive(Debug)]
pub struct ActionKV {
    f: File,
    pub index: HashMap<ByteString, u64>,

}

impl ActionKV {
    pub fn open(path: &Path) -> io::Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;
        let index = HashMap::new();
        Ok( ActionKV { f, index } )

    }

    pub fn load(&mut self) -> io::Result<()> {
        let mut f = BufReader::new(&self.f);
        loop {
            let position = f.seek(SeekFrom::Current(0))?;
            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(err) => {
                    match err.kind() {
                        io::ErrorKind::UnexpectedEof => { break; }
                        _ => return Err(err),
                    }
                }
            };
            self.index.insert(kv.key, position);
        };
        Ok(())
    }

    pub fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
        let saved_checksum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let val_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len + val_len;

        let mut data = ByteString::with_capacity(data_len as usize);
        {
            f.by_ref()
                .take(data_len as u64)
                .read_to_end(&mut data)?;
        }
        debug_assert_eq!(data.len(), data_len as usize);

        let check_sum = crc32::checksum_ieee(&data);
        if check_sum != saved_checksum {
            panic!("data corruption encountered ({} != {})", check_sum, saved_checksum);
        }
        let value = data.split_off(key_len as usize);
        let key = data;
        Ok(KeyValuePair { key, value })
    }

    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) ->io::Result<()> {
        let position = self.insert_but_ignore_index(key, value)?;
        self.index.insert(key.to_vec(), position);
        Ok(())
    }


    pub fn insert_but_ignore_index(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<u64> {
        let mut f = BufWriter::new(&mut self.f);

        let key_len = key.len();
        let val_len = value.len();
        let mut tmp = ByteString::with_capacity(key_len + val_len);

        for byte in key {
            tmp.push(*byte);
        }

        for byte in value {
            tmp.push(*byte);
        }

        let checksum = crc32::checksum_ieee(&tmp);
        let next_byte = SeekFrom::End(0);
        let current_position = f.seek(SeekFrom::Current(0))?;
        f.seek(next_byte)?;
        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key_len as u32)?;
        f.write_u32::<LittleEndian>(val_len as u32)?;
        f.write_all(&tmp)?;
        Ok(current_position)

    }

    pub fn get(&mut self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        let position = match self.index.get(key) {
            None => return Ok(None),
            Some(position) => *position,
        };
        let kv = self.get_at(position)?;
        Ok(Some(kv.value))
    }

    pub fn get_at(&mut self, position: u64) -> io::Result<KeyValuePair> {
        let mut f = BufReader::new(&mut self.f);
        f.seek(SeekFrom::Start(position))?;
        let kv = ActionKV::process_record(&mut f)?;
        Ok(kv)
    }

    pub fn find(&mut self, target: &ByteStr) -> io::Result<Option<(u64, ByteString)>> {
        let mut f = BufReader::new(&self.f);
        let mut found: Option<(u64, ByteString)> = None;
        f.seek(SeekFrom::Start(0))?;
        loop {
            let position = f.seek(SeekFrom::Current(0))?;
            let maybe_kv = match ActionKV::process_record(&mut f) {
                Ok(kv) => kv,
                Err(err) => {
                    match err.kind() {
                        io::ErrorKind::UnexpectedEof => {
                            break;
                        }
                        _ => return Err(err),
                    }
                }
            };
            if maybe_kv.key == target {
                found = Some((position, maybe_kv.value));
            }
        }
        Ok(found)
    }

    pub fn update(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()>{
        self.insert(key, value)
    }

    pub fn delete(&mut self, key: &ByteStr) -> io::Result<()> {
        self.insert(key, b"")
    }
}


#[cfg(test)]
mod test {
    use super::ActionKV;
    use std::{fs, ffi::OsStr, path::Path};

    fn init_db<P: AsRef<OsStr>>(fname: P) -> ActionKV {
        let path = std::path::Path::new(&fname);
        let mut store = ActionKV::open(path).expect("unable to open file");
        store.load().expect("unable to load data");
        store
    }

    fn clear_db<P: AsRef<Path>>(fname: P) -> std::io::Result<()>{
        fs::remove_file(fname)?;
        Ok(())
    }


    #[test]
    fn test_get_action() {
        let fname = "test_get";
        let mut store = init_db(fname);
        let key = "apple";
        let value = "100";
        store.insert(key.as_bytes(), value.as_bytes()).unwrap();
        store.load().expect("unable to load data");
        let value_from_db = store.get(key.as_bytes()).unwrap();
        assert_eq!(value_from_db.unwrap(), value.as_bytes());
        clear_db(fname).unwrap();
    }

    #[test]
    fn test_find_action() {
        let fname = "test_find";
        let mut store = init_db(fname);
        let key = "apple";
        let value = "100";
        store.insert(key.as_bytes(), value.as_bytes()).unwrap();
        store.load().expect("unable to load data");
        let value_from_db = store.find(key.as_bytes()).unwrap();
        let pos = store.index.get(key.as_bytes()).unwrap();
        assert_eq!(value_from_db.unwrap(), (*pos, value.as_bytes().to_vec()));
        clear_db(fname).unwrap();
    }
}
