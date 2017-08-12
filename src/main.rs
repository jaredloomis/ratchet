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
mod blockchain;

use blockchain::{BlockchainServer, BlockData};

fn main() {
    // Get port from args
    let args: Vec<_> = env::args().collect();
    let def_port     = String::from("4567");
    let port         = args.get(1).unwrap_or(&def_port);

    let mut server: BlockchainServer<BlockData> = BlockchainServer::new();
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
