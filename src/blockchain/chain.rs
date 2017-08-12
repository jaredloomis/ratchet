use std::fmt;
use std::io::Read;

use serde::{Serialize, Deserialize};
use serde_json;

use reqwest;

use blockchain::block::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockchainNode<T> {
    pub blockchain: Blockchain<T>,
    pub transactions: Vec<Transaction>,
    pub peers: Vec<String>
}

pub type Blockchain<T> = Vec<Block<T>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockData {
    proof_of_work: u64,
    transactions: Vec<Transaction>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from:   Hash,
    pub to:     Hash,
    pub amount: u64
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction({}, {} -> {})",
            self.amount, self.from, self.to
        )
    }
}

impl BlockchainNode<BlockData> {
    pub fn new() -> BlockchainNode<BlockData> {
        BlockchainNode {
            blockchain:   vec![BlockchainNode::genesis_block()],
            transactions: Vec::new(),
            peers:        Vec::new()
        }
    }

    pub fn add_peer(&mut self, peer: String) {
        self.peers.push(peer);
    }

    pub fn balance(&self, address: &Hash) -> u64 {
        // Starting from 0,
        // Loop through each block
        let tmp_tot = self.blockchain.iter().fold(0, |bal_i, block| {
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
        });
        // Add up pending transactions
        self.transactions.iter().fold(tmp_tot, |bal, trans| {
            if trans.from == *address {
                bal - trans.amount
            } else if trans.to == *address {
                bal + trans.amount
            } else {
                bal
            }
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
                    last_index + 1, primitive_timestamp(),
                    new_block_data, last_hash.clone()
                );
                self.blockchain.push(mined_block);
                self.blockchain.last()
            },
            None => None
        }
    }

    pub fn consensus(&mut self) {
        // Get the blocks from other nodes
        let other_chains = self.find_new_chains();
        println!("OTHER CHAINS: {}", other_chains.len());
        // If our chain isn't longest,
        // then we store the longest chain
        let bc = &self.blockchain.clone();
        let longest_chain = other_chains.iter().fold(bc, |best, cur| {
            println!("Cur: {:?}", cur);
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
            let mut block_res    = reqwest::get(format!("http://{}/blocks", node_url).as_str());
            if block_res.is_ok() {
                let mut block_string = String::new();
                block_res.unwrap().read_to_string(&mut block_string);
                let block = serde_json::from_str(block_string.as_str());
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

