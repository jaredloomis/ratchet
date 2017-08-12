use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::LinkedList;
use std::io::Read;

use chrono::{DateTime, Utc};
use crypto::sha2::Sha512;
use crypto::digest::Digest;

use serde::Serialize;
use serde_json;
use serde_json::{Value, Error};

use reqwest;

use http::parser::{HttpRequest, HttpResponse};

pub type Blockchain<T> = Vec<Block<T>>;

pub fn blockchain() -> Blockchain<String> {
    let size = 10;
    let mut chain: Blockchain<String> = Vec::with_capacity(size);
    chain.push(Block::genesis());
    for i in 0..(size-1) {
        let next_block_m = chain.last_mut().map(|tip| tip.next_block("some".into()));
        // borrow of `chain` ended
        match next_block_m {
            Some(next_block) => chain.push(next_block),
            None => {}
        }
    }
    chain
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block<T> {
    index:         u64,
    timestamp:     Instant,
    data:          T,
    hash:          Hash,
    previous_hash: Hash
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

fn primitive_timestamp() -> Instant {
    Utc::now()
}

fn primitive_hash(index: &u64, timestamp: &Instant, data_str: &String, previous_hash: &Hash) -> Hash {
    let mut hasher = Sha512::new();
    let det_str = format!("{}{}{}{}", index, timestamp, data_str, previous_hash);
    hasher.input_str(det_str.as_str());
    hasher.result_str()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    from:   Hash,
    to:     Hash,
    amount: u64
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction({}, {}, {})",
            self.from, self.to, self.amount
        )
    }
}

/*
 * Server
 */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockchainServer<T> {
    blockchain: Blockchain<T>,
    transactions: Vec<Transaction>,
    peers: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockData {
    proof_of_work: u64,
    transactions: Vec<Transaction>
}

impl BlockchainServer<BlockData> {
    pub fn new() -> BlockchainServer<BlockData> {
        BlockchainServer {
            blockchain:   vec![BlockchainServer::genesis_block()],
            transactions: Vec::new(),
            peers:        Vec::new()
        }
    }

    pub fn handle_req(&mut self, req: HttpRequest) -> Option<HttpResponse> {
        if req.path == "/transaction" && req.method == "POST" {
            // On each new POST request,
            // we extract the transaction data
            // Then we add the transaction to our list
            match serde_json::from_str(req.body.as_str()) {
                Ok(value) => self.transactions.push(value),
                _         => {}
            }

            match serde_json::to_string(&self.transactions) {
                Ok(transactions_json) =>
                    Some(HttpResponse::new(
                        String::from("200 OK"), Vec::new(), transactions_json
                    )),
                _ => None
            }
        } else if req.path.starts_with("/mine") && req.method == "GET" {
            let address_m = req.params.get("address");
            address_m.and_then(|address| self.mine(address.clone())).and_then(|block| {
                match serde_json::to_string(&block) {
                    Ok(block_json) =>
                        Some(HttpResponse::new(
                            String::from("200 OK"), Vec::new(), block_json
                        )),
                    _ => None
                }
            })
        } else if req.path.starts_with("/consensus") {
            // Achieve consensus
            self.consensus();
            // Send blockchain
            match serde_json::to_string(&self.blockchain) {
                Ok(blockchain_json) =>
                    Some(HttpResponse::new(
                        String::from("200 OK"), Vec::new(), blockchain_json
                    )),
                _ => None
            }
        } else if req.path.starts_with("/blocks") {
            // Send blockchain
            match serde_json::to_string(&self.blockchain) {
                Ok(blockchain_json) =>
                    Some(HttpResponse::new(
                        String::from("200 OK"), Vec::new(), blockchain_json
                    )),
                _ => None
            }
        } else if req.path.starts_with("/balance") {
            req.params.get("address")
                .map(|address| self.balance(address))
                .map(|balance| HttpResponse::new(
                    String::from("200 OK"), Vec::new(), balance.to_string()
                ))
        } else {
            None
        }
    }

    pub fn balance(&self, address: &Hash) -> u64 {
        // Starting from 0,
        // Loop through each block
        self.blockchain.iter().fold(0, |bal_i, block| {
            // And each transaction
            block.data.transactions.iter().fold(bal_i, |bal, trans| {
                // Add up each transaction
                if trans.from == *address {
                    bal - trans.amount
                } else if trans.to == *address {
                    bal + trans.amount
                } else {
                    bal
                }
            })
        })
    }

    pub fn mine(&mut self, miner_address: Hash) -> Option<&Block<BlockData>> {
        let last_m = self.blockchain.last().map(|last| {
            (last.index, last.hash.clone(), last.data.clone())
        });
        match last_m {
            Some((last_index, last_hash, last_data)) => {
                let proof = proof_of_work(last_data.proof_of_work);
                // Once we find a valid proof of work,
                // we know we can mine a block so 
                // we reward the miner by adding a transaction
                self.transactions.push(Transaction {
                    from: String::from("network"),
                    to: miner_address,
                    amount: 1
                });
                // Now we can gather the data needed
                // to create the new block
                let new_block_data = BlockData {
                    proof_of_work: proof,
                    transactions: self.transactions.clone()
                };
                // Empty transaction list
                self.transactions.clear();
                // Add block to blockchain
                let mined_block = Block::new(
                    last_index + 1, Utc::now(),
                    new_block_data, last_hash.clone()
                );
                self.blockchain.push(mined_block);
                self.blockchain.last()
                //Some(mined_block)
            },
            None => None
        }
    }

    pub fn consensus(&mut self) {
        // Get the blocks from other nodes
        let other_chains = self.find_new_chains();
        // If our chain isn't longest,
        // then we store the longest chain
        let bc = &self.blockchain.clone();
        let longest_chain = other_chains.iter().fold(bc, |best, cur| {
            if cur.len() > best.len() {
                cur
            } else {
                best
            }
        });
        // If the longest chain wasn't ours,
        // then we set our chain to the longest
        self.blockchain = longest_chain.clone();
    }

    fn find_new_chains(&self) -> Vec<Blockchain<BlockData>> {
        // Get the blockchains of every
        // other node
        let mut other_chains = Vec::new();
        for node_url in self.peers.clone() {
            // Get their chains using a GET request
            let mut block_res    = reqwest::get("/blocks");
            let mut block_string = String::new();
            if block_res.is_ok() {
                block_res.unwrap().read_to_string(&mut block_string);
                let block: Result<Blockchain<BlockData>, Error> = serde_json::from_str(block_string.as_str()); 
                // Add it to our list
                if block.is_ok() {
                    other_chains.push(block.unwrap());
                }
            }
        }
        other_chains
    }

    fn genesis_block() -> Block<BlockData> {
        let data = BlockData {
            proof_of_work: 1,
            transactions: Vec::new()
        };
        Block::new(0, primitive_timestamp(), data, String::from("0"))
    }
}

pub fn proof_of_work(last_proof: u64) -> u64 {
    // Create a variable that we will use to find
    // our next proof of work
    let mut incrementor = last_proof + 1;
    // Keep incrementing the incrementor until
    // it's equal to a number divisible by 9
    // and the proof of work of the previous
    // block in the chain
    while !(incrementor % 9 == 0 && incrementor % last_proof == 0) {
        incrementor += 1
    }
    // Once that number is found,
    // we can return it as a proof
    // of our work
    incrementor
}


