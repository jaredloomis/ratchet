use std;
use std::io::Read;
use std::collections::HashSet;

use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_json;

use reqwest;

use blockchain::block::*;

/**
 * A node on the blockchain. Client and server
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockchainNode<T> {
    pub blockchain: Blockchain<T>,
    pub transactions: Vec<Transaction>,
    pub peers: HashSet<String>
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

/**
 * A Blockchain is a list of blocks
 */
pub type Blockchain<T> = Vec<Block<T>>;

/**
 * If the data inside blocks contains transactions,
 * we can do a bunch of cool stuff generically
 */
pub trait HasTransactions {
    fn transactions(&self) -> &Vec<Transaction>;
}

impl<T> BlockchainNode<T> where T: HasTransactions + Serialize + DeserializeOwned + Clone {
    /**
     * Create a blockchain node with a single genesis block
     */
    pub fn new(genesis_block: Block<T>) -> BlockchainNode<T> {
        BlockchainNode {
            blockchain:   vec![genesis_block],
            transactions: Vec::new(),
            peers:        HashSet::new()
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
            block.data.transactions().iter().fold(bal_i, |bal, trans| {
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
        // Get the chains from other nodes
        let other_chains = self.find_new_chains();
        // Find longest chain
        let bc = &self.blockchain.clone();
        let longest_chain = other_chains.iter().fold(bc, |best, cur| {
            if cur.len() > best.len() {
                cur
            } else {
                best
            }
        });
        // If our chain isn't longest, store longest chain
        self.blockchain = longest_chain.clone();
    }

    /**
     * Collect all the other chains peers have
     */
    fn find_new_chains(&self) -> Vec<Blockchain<T>> {
        let mut other_chains = Vec::new();
        for node_url in self.peers.clone() {
            // Get the blockchains of peers using a GET request
            let mut chain_res    = reqwest::get(format!("http://{}/blocks", node_url).as_str());
            if chain_res.is_ok() {
                // Parse response into a block
                let mut chain_string = String::new();
                chain_res.unwrap().read_to_string(&mut chain_string);
                let chain = serde_json::from_str(chain_string.as_str());
                // Add it to our list
                if chain.is_ok() {
                    other_chains.push(chain.unwrap());
                }
            }
        }
        other_chains
    }

    /**
     * Add peers of current peers to tracked peers
     */
    fn find_new_peers(&mut self) {
        for peer in self.peers_of_peers() {
            self.peers.insert(peer);
        }
    }

    /**
     * Ask all of my peers to send me their peers
     */
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

}
