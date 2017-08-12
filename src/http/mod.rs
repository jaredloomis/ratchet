#[macro_use]

use std::option::Option;
use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::fmt::format;
use std::str;
use std::time::Duration;

use nom::*;
use nom::IResult::*;

pub mod parser;
use http::parser::{parse_request, HttpResponse, HttpRequest};

fn read_to_string_chunks(mut stream: &TcpStream) -> String {
    let mut ret = String::new();
    let mut buf: [u8; 1024] = [0; 1024];
    let mut read_bytes = 1;
    while read_bytes > 0 {
        match stream.read(&mut buf) {
            Ok(bytes) => {read_bytes = bytes;}
            _         => {read_bytes = 0;}
        }
        let mbuf_str = str::from_utf8(&buf);
        if mbuf_str.is_ok() {
            ret.push_str(mbuf_str.unwrap());
        }
    }
    ret
}

fn get_request(mut stream: &TcpStream) -> Option<HttpRequest> {
    let mut httpRequest = String::new();
    println!("Reading from stream...");
    //let mut httpRequest  = [0; 1024];
    let bytes_read = stream.read_to_string(&mut httpRequest);
    //let httpRequest = read_to_string_chunks(&stream);

    println!("Starting parsing {}...", httpRequest);
    match parse_request(httpRequest.as_bytes()) {
        Done(i, req)    => {
            println!("Parser success: {:?}", req);
            Some(req)
        },
        Error(err)      => {
            println!("Parser err: {}", err);
            None
        },
        Incomplete(inc) => {
            println!("Parse incomplete: {:?}", inc);
            None
        }
    }
}

fn send_response(mut stream: &TcpStream, res: &HttpResponse) {
    let _ = stream.write(res.to_string().as_bytes());
}

pub fn start_server(address: &String, mut handler: Box<FnMut(HttpRequest) -> Option<HttpResponse>>) {
    let listener = TcpListener::bind(address).unwrap();
    println!("Listening on {}...", address);

    for streamRes in listener.incoming() {
        match streamRes {
            Ok(mut stream) => {
                println!("Got connection!");
                stream.set_read_timeout(Some(Duration::new(1, 0)));
                get_request(&stream)
                    .and_then(|req| handler(req))
                    .map(|res| send_response(&stream, &res));
            },
            Err(err) => println!("ERROR in incoming! {}", err)
        };
    }

    println!("Exiting...");
}
