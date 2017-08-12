use std::io::prelude::*;
use std::fmt::format;
use std::str;
use std::collections::HashMap;
use std::iter::Iterator;

extern crate nom;
use nom::*;
use nom::IResult::*;

#[derive(Debug)]
pub struct HttpRequest {
    pub method:  String,
    pub path:    String,
    pub params:  HashMap<String, String>,
    pub headers: Vec<String>,
    pub body:    String
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status:  String,
    pub headers: Vec<String>,
    pub body:    String
}

impl HttpResponse {
    pub fn new(status: String, headers: Vec<String>, body: String) -> HttpResponse {
        HttpResponse {
            status: status,
            headers: headers,
            body: body
        }
    }

    pub fn to_string(&self) -> String {
        let headers = self.headers.join("\r\n");
        format!("HTTP/1.1 {}\r\n{}\r\n{}", self.status, headers, self.body)
    }
}

pub fn utf8_to_string(bytes: &[u8]) -> String {
    let vector: Vec<u8> = Vec::from(bytes);
    String::from_utf8(vector).unwrap()
}

fn is_end_of_path(c: u8) -> bool {
    c == ('?'  as u8) || c == (' '  as u8) ||
    c == ('\r' as u8) || c == ('\n' as u8)
}

fn is_end_of_param(c: u8) -> bool {
    c == ('&'  as u8) || c == (' '  as u8) ||
    c == ('\r' as u8) || c == ('\n' as u8) ||
    c == ('='  as u8)
}

named!(identifier_tail(&[u8]) -> String, map!(many0!(none_of!("\n\r ?=&")), |vec| {
    let mut ret = String::new();
    for c in vec {
        ret.push(c);
    }
    ret
}));

named!(identifier(&[u8]) -> String, do_parse!(
    head: alpha           >>
    tail: identifier_tail >>
    ({
        let mut ret = utf8_to_string(head);
        ret.push_str(tail.as_str());
        ret
    })
));

named!(method, alt!(tag!("GET") | tag!("POST")));
named!(path, take_till!(is_end_of_path));
named!(param(&[u8]) -> (String, Option<String>), do_parse!(
    name:  identifier       >>
    opt!(tag!("="))         >>
    value: opt!(identifier) >>
    (name, value)
));
named!(params(&[u8]) -> HashMap<String, String>, map!(many0!(param), |vec| {
    let mut ret = HashMap::new();
    for (key, val) in vec {
        ret.insert(key, val.unwrap_or(String::from("")));
    }
    ret
}));
named!(header, do_parse!(
    ret: take_until!("\r\n") >>
    tag!("\r\n") >>
    (ret)
));
named!(body, take_till!(call!(|_| false)));
named!(pub parse_request(&[u8]) -> HttpRequest, do_parse!(
    meth: ws!(method)      >>
    pth:  path             >>
    opt!(tag!("?"))        >>
    prms: params           >>
    ws!(tag!("HTTP/1.1"))  >>
    hdrs: many0!(header)   >>
    bdy:  body             >>
    (HttpRequest {
        method: utf8_to_string(meth),
        path: utf8_to_string(pth),
        params: prms,
        headers: hdrs.iter().map(|bytes| utf8_to_string(bytes)).collect(),
        body: utf8_to_string(bdy)
    })
));

fn concat_lines(lines: Vec<&[u8]>) -> String {
    let mut ret = String::new();
    for line in lines {
        let line_str_m = str::from_utf8(line);
        if line_str_m.is_ok() {
            ret.push_str(line_str_m.unwrap());
        }
    }
    ret
}
