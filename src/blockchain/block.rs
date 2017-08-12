use std::fmt;

use crypto::sha2::Sha512;
use crypto::digest::Digest;

use chrono::{Utc, DateTime};

use serde::Serialize;
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block<T> {
    pub index:         u64,
    pub timestamp:     Instant,
    pub data:          T,
    pub hash:          Hash,
    pub previous_hash: Hash
}

pub type Instant = DateTime<Utc>;
pub type Hash = String;

impl Block<String> {
    pub fn genesis() -> Block<String> {
        Block::new(0, primitive_timestamp(), String::from("Genesis Block"), String::from("0"))
    }
}

impl<T> Block<T> where T: Serialize {
    pub fn new(index: u64, timestamp: Instant, data: T, previous_hash: Hash) -> Block<T> {
        let data_str: String = match serde_json::to_string(&data) {
                Ok(ret) => ret,
                _       => String::new()
        };
        let hash = primitive_hash(
            &index, &timestamp, &data_str, &previous_hash
        );

        Block {
            index: index,
            timestamp: timestamp,
            data: data,
            hash: hash,
            previous_hash: previous_hash
        }
    }

    pub fn next_block(&self, data: T) -> Block<T> {
        Block::new(self.index+1, primitive_timestamp(), data, self.hash.clone())
    }
}

impl<T> fmt::Display for Block<T> where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "Block({}, {}, {}, {}, {})",
            self.index, self.timestamp, self.data, self.hash, self.previous_hash
        )
    }
}

pub fn primitive_timestamp() -> Instant {
    Utc::now()
}

pub fn primitive_hash(index: &u64, timestamp: &Instant, data_str: &String, previous_hash: &Hash) -> Hash {
    let mut hasher = Sha512::new();
    let det_str = format!("{}{}{}{}", index, timestamp, data_str, previous_hash);
    hasher.input_str(det_str.as_str());
    hasher.result_str()
}
