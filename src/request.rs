use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};
use std::io::Read;
use std::str::FromStr;

use crate::header::HttpHeader;
use crate::errors::ParsingError;

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl TryFrom<String> for HttpMethod {
    type Error = ParsingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl FromStr for HttpMethod {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(Self::GET),
            "post" => Ok(Self::POST),
            "put" => Ok(Self::PUT),
            "delete" => Ok(Self::DELETE),
            _ => Err(ParsingError::UnsupportedOrMissingMethodError(s.to_string()))
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub headers: Vec<HttpHeader>,
    pub body: Option<String>,
}

impl Request {
    pub fn try_read(stream: &mut TcpStream) -> Result<Self, anyhow::Error> {
        let mut buffer = BufReader::new(stream);

        // Read request line
        let mut line = String::new();
        buffer.read_line(&mut line)?;

        let (method, path) = parse_method_path(&line)?;
        line.clear();

        // Parsing headers.
        // Headers and the body are separated by additional "\r\n", so searching for that.
        let mut headers: Vec<HttpHeader> = vec![];
        loop {
            buffer.read_line(&mut line)?;

            if line.starts_with("\r\n") {
                line.clear();
                break
            }

            if let Some(header) = HttpHeader::try_from(line.as_str()).ok() {
                headers.push(header);
            }

            line.clear();
        }

        // Next, read the body (if present).
        // Since there might be that no EOF or new-line symbols present, the only way to correctly
        // read the body is by utilizing the Content-Length header.
        let content_length = headers.iter()
            .find(|header| header.name.to_lowercase() == "content-length")
            .and_then(|header| header.value.parse::<usize>().ok());

        let body: Option<String> = match content_length {
            Some(len) => {
                let mut body_buff = vec![0; len];
                buffer.read_exact(&mut body_buff)?;
                String::from_utf8(body_buff).ok()
            },
            None => None
        };

        Ok(Request {
            method,
            path,
            headers,
            body,
        })
    }
}

fn parse_method_path(line: &str) -> Result<(HttpMethod, String), ParsingError> {
    let mut parts = line.split(' ');
    let method = parts.next()
        .map(HttpMethod::from_str)
        .ok_or(ParsingError::UnsupportedOrMissingMethodError(String::new()))??;
    let path = parts.next().ok_or(ParsingError::NoPathError)?;

    Ok((method, path.to_string()))
}