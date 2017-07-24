#[macro_use]

use std::option::Option;
use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::fmt::format;

extern crate nom;
use nom::*;
use nom::IResult::*;

/*
 * Help
 */

fn utf8_to_string(bytes: &[u8]) -> String {
    let vector: Vec<u8> = Vec::from(bytes);
    String::from_utf8(vector).unwrap()
}

/*
 * Structure
 */

struct HttpRequest {
    method:  String,
    path:    String,
    headers: Vec<String>
}

struct HttpResponse {
    status:  String,
    headers: Vec<String>,
    body:    String
}

impl HttpResponse {
    fn to_string(&self) -> String {
        let headers = self.headers.join("\r\n");
        format!("HTTP/1.1 {}\r\n{}\r\n{}", self.status, headers, self.body)
    }
}

/*
 * Parser
 */

named!(method, alt!(tag!("GET") | tag!("POST")));

named!(path, take_until!(" "));

named!(header, take_until!("\n"));

named!(request(&[u8]) -> HttpRequest, do_parse!(
    meth: ws!(method)         >>
    pth:  ws!(path)           >>
    ws!(tag!("HTTP/1.1"))     >>
    hdrs: ws!(many0!(header)) >>
    (HttpRequest {
        method: utf8_to_string(meth),
        path: utf8_to_string(pth),
        headers: hdrs.iter().map(|bytes| utf8_to_string(bytes)).collect()
    })
));

/*
 * Handler
 */

fn user_handler(path: &String) -> Option<HttpResponse> {
    if path == "/party" {
        Some(HttpResponse {
            status: String::from("200 OK"),
            headers: Vec::new(),
            body: String::from("LETS PARTY WHOo")
        })
    } else {
        None
    }
}

fn get_path(path: &String) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn generate_response(req: &HttpRequest) -> HttpResponse {
    let user_resp = user_handler(&req.path);
    user_resp.unwrap_or_else(|| match get_path(&req.path) {
        Ok(body) => HttpResponse {
            status: String::from("200 OK"),
            headers: Vec::new(),
            body: body
        },
        Err(_)  => HttpResponse {
            status: String::from("404 Not Found"),
            headers: Vec::new(),
            body: String::new()
        }
    })
}

fn handle_client(stream: &mut TcpStream) {
    // Read request from stream
    let mut httpRequest  = [0; 1024];
    let readBytes = stream.read(&mut httpRequest);
    match readBytes {
        Ok(bytes) => println!("Read {} bytes", bytes),
        Err(err)  => println!("Err reading req {}", err)
    }
    println!("Request:\n{}", utf8_to_string(&httpRequest));
    // Parse request into HttpRequest
    let mreq = request(&httpRequest);
    match mreq {
        Done(i, req) => {
            // Generate a response
            let res = generate_response(&req).to_string();
            // Send response
            let _ = stream.write(res.as_bytes());
        },
        Error(err)    => println!("Got Err {}", err),
        Incomplete(_) => println!("Got Incomplete ?")
    }
}

fn main() {
    let address  = "127.0.0.1:4567";
    let listener = TcpListener::bind(address).unwrap();

    println!("Listening on {}...", address);

    for streamRes in listener.incoming() {
        match streamRes {
            Ok(mut stream) => handle_client(&mut stream),
            Err(err)       => println!("ERROR in incoming! {}", err)
        };
    }

    println!("Exiting...");
}
