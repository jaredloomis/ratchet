use std::fmt;
use std::io::Read;

use serde::{Serialize, Deserialize};
use serde_json;

use reqwest;

use blockchain::block::*;

/**
 * A node in the blockchain. Client and server
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockchainNode<T> {
    pub blockchain: Blockchain<T>,
    pub transactions: Vec<Transaction>,
    pub peers: Vec<String>
}

/**
 * A Blockchain is a list of blocks
 */
pub type Blockchain<T> = Vec<Block<T>>;

/**
 * Data being stored in the ledger:
 * - Proof of work (initial value)
 * - Transactions  (moving around value)
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockData {
    proof_of_work: u64,
    transactions: Vec<Transaction>
}

/**
 * An amount being exchanged between two nodes
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from:   Hash,
    pub to:     Hash,
    pub amount: u64
}

impl BlockchainNode<BlockData> {
    /**
     * Create a blockchain node with a single genesis block
     */
    pub fn new() -> BlockchainNode<BlockData> {
        BlockchainNode {
            blockchain:   vec![BlockchainNode::genesis_block()],
            transactions: Vec::new(),
            peers:        Vec::new()
        }
    }

    /**
     * Get the balance associated with an address
     */
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

    /**
     * Mine a block
     */
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

    /**
     * Verify integrity of entire chain
     */
    pub fn verify(&self) -> bool {
        self.blockchain.iter().all(|block| block.verify())
    }

    /**
     * Check all other nodes for a more valid blockchain
     */
    pub fn consensus(&mut self) {
        // Ask neighbors for new peers
        self.find_new_peers();
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

    /**
     * Add peers of current peers to tracked peers
     */
    fn find_new_peers(&mut self) {
        for peer in self.peers_of_peers() {
            self.peers.push(peer);
        }
    }

    /**
     * Collect all the other chains peers have
     */
    fn find_new_chains(&self) -> Vec<Blockchain<BlockData>> {
        let mut other_chains = Vec::new();
        for node_url in self.peers.clone() {
            // Get the blockchains of peers using a GET request
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

    fn peers_of_peers(&self) -> Vec<String> {
        self.peers.iter().filter_map(|peer| {
            let peers_res = reqwest::get(format!("http://{}/peers", peer).as_str());
            if peers_res.is_ok() {
                let mut peers_string = String::new();
                peers_res.unwrap().read_to_string(&mut peers_string);
                // Parse
                let peers = serde_json::from_str(peers_string.as_str());
                // Add it to our list
                if peers.is_ok() {
                    Some(peers.unwrap())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
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

