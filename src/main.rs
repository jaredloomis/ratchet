#[macro_use]

use std::option::Option;
use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::fmt::format;
use std::env;

extern crate nom;
use nom::*;
use nom::IResult::*;

extern crate regex;
extern crate chrono;
extern crate crypto;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate reqwest;

mod http;
use http::parser::{HttpResponse, HttpRequest};
mod blockchain;
use blockchain::chain::{BlockchainNode};
use blockchain::transaction_chain::{TransactionBlockData};

impl BlockchainNode<TransactionBlockData> {
    pub fn handle_req(&mut self, req: HttpRequest) -> Option<HttpResponse> {
        if req.path == "/transaction" && req.method == "POST" {
            // On each new POST request,
            // we extract the transaction data
            // Then we add the transaction to our list
            serde_json::from_str(req.body.as_str()).ok().map(move |trans| {
                let res = HttpResponse::new(
                    String::from("200 OK"), Vec::new(), req.body
                );
                self.transactions.push(trans);
                res
            })
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
        } else if req.path.starts_with("/connect") {
            req.params.get("peer")
                .and_then(|peer| {
                    self.peers.insert(peer.clone());
                    self.consensus();
                    serde_json::to_string(&self.peers).map(|peers_str| {
                        HttpResponse::new(
                            String::from("200 OK"), Vec::new(), peers_str
                        )
                    }).ok()
                })
        } else if req.path.starts_with("/peers") {
            serde_json::to_string(&self.peers).map(|peers_str| {
                HttpResponse::new(
                    String::from("200 OK"), Vec::new(), peers_str
                )
            }).ok()
        } else if req.path.starts_with("/verify") {
            Some(HttpResponse::new(
                String::from("200 OK"), Vec::new(), self.verify().to_string()
            ))
        } else {
            None
        }
    }
}

fn main() {
    // Get port from args
    let args: Vec<_> = env::args().collect();
    let def_port     = String::from("4567");
    let port         = args.get(1).unwrap_or(&def_port);

    let genesis = BlockchainNode::<TransactionBlockData>::genesis_block();
    let mut server: BlockchainNode<TransactionBlockData> = BlockchainNode::new(genesis);
    for i in 0..10 {
        server.mine(String::from("jaredloomis"));
    }

    http::start_server(&String::from(format!("localhost:{}", port)), Box::new(move |req| {
        println!("GOT REQ {:?}", req);
        let ret = server.handle_req(req);
        println!("HANDLED REQ {:?}", ret);
        ret
    }));
}
