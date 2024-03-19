// Uncomment this block to pass the first stage
use std::{
    env, fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use anyhow::anyhow;
use crate::header::HttpHeader;
use crate::request::{HttpMethod, Request};

mod request;
mod errors;
mod header;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let directory = extract_directory();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let dir = directory.clone();
                thread::spawn(move || process(stream, dir));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn extract_directory() -> Option<String> {
    for (i, arg) in env::args().enumerate() {
        if arg == "--directory" {
            return env::args().nth(i + 1);
        }
    }

    None
}

fn read_file(dir: Option<String>, filename: &str) -> Option<String> {
    let path = dir.map(|d| {
        let delimiter = if d.ends_with('/') { "" } else { "/" };
        format!("{}{}{}", d, delimiter, filename)
    });

    match path {
        Some(p) => fs::read_to_string(p).ok(),
        None => None,
    }
}

fn read_headers(buffer: &mut BufReader<&TcpStream>) -> String {
    let mut data = String::new();

    let mut line = String::new();
    loop {
        buffer.read_line(&mut line).unwrap_or_default();
        if line.starts_with("\r\n") {
            break;
        }

        data.push_str(&line);
        line.clear();
    }
    data
}

fn read_body(buffer: &mut BufReader<&TcpStream>, len: usize) -> String {
    let mut body_bytes = vec![0; len];

    buffer.read_exact(&mut body_bytes).unwrap_or_default();

    String::from_utf8(body_bytes).unwrap_or(String::new())
}

fn process(mut stream: TcpStream, dir: Option<String>) {
    println!("accepted new connection");
    
    let request = Request::try_read(&mut stream);
    
    let response = match request { 
        Ok(req) => handle_request(req, dir).unwrap_or_else(Response::error),
        Err(e) => Response::error(e),
    }; 
    
    response.write_to(&mut stream);
}

fn handle_request(req: Request, dir: Option<String>) -> Result<Response, anyhow::Error> {
    let path = req.path.strip_suffix('/').unwrap_or(req.path.as_str());
    let method = &req.method;

    match (method, path) {
        (HttpMethod::GET, "/") => Response::empty_ok(),

        (HttpMethod::GET, "/user-agent") => {
            let user_agent = &req.headers.iter()
                .find(|header| header.name.to_lowercase() == "user-agent")
                .map_or("None".to_string(), |header| header.value.to_owned());
            let headers = vec![
                format!("Content-Length: {}", user_agent.len()),
                format!("Content-Type: {}", "text/plain")
            ];
            Response::ok(headers, Some(user_agent.to_owned()))
        }

        (HttpMethod::GET, path) if path.starts_with("/echo/") => {
            let echo = &path[6..];
            let headers = vec![
                format!("Content-Length: {}", echo.len()),
                format!("Content-Type: {}", "text/plain")
            ];
            Response::ok(headers, Some(echo.to_string()))
        }

        (HttpMethod::GET, path) if path.starts_with("/files/") => {
            let filename = &path[7..];
            let file_res = read_file(dir, filename);

            match file_res {
                Some(content) => {
                    let headers: Vec<String> = vec![
                        format!("Content-Length: {}", content.len()),
                        format!("Content-Type: {}", "application/octet-stream")
                    ];
                    Response::ok(headers, Some(content))
                }
                None => Response::not_found(),
            }
        }

        (HttpMethod::POST, path) if path.starts_with("/files/") => {
            let filename = &path[7..];
            let write_path = dir.map(|d| format!("{}/{}", d, filename)).unwrap();

            let write_result = fs::write(write_path, req.body.unwrap_or("".to_string()));
            if write_result.is_ok() {
                Response::created()
            } else {
                Response::not_found()
            }
        }

        (_, _) => Response::not_found(),
    };
    Err(anyhow!(""))
}

struct Response {
    status_code: (usize, String),
    headers: Vec<String>,
    body: Option<String>,
}

impl Response {
    fn as_string(&self) -> String {
        let headers_str = self.headers.join("\r\n");
        let status_str = format!("HTTP/1.1 {} {}", self.status_code.0, self.status_code.1);
        let body_str = match &self.body {
            Some(b) => b.as_str(),
            None => "",
        };

        format!("{}\r\n{}\r\n\r\n{}", status_str, headers_str, body_str)
    }

    fn ok(headers: Vec<String>, body: Option<String>) -> Response {
        Response {
            status_code: (200, String::from("Ok")),
            headers,
            body,
        }
    }

    fn empty_ok() -> Response {
        Response {
            status_code: (200, String::from("OK")),
            headers: vec![],
            body: None,
        }
    }

    fn created() -> Self {
        Response {
            status_code: (201, String::from("Created")),
            headers: vec![],
            body: None,
        }
    }

    fn not_found() -> Response {
        Response {
            status_code: (404, String::from("Not found")),
            headers: vec![],
            body: None,
        }
    }
    
    fn error(e: anyhow::Error) -> Self {
        Response {
            status_code: (500, "ERROR".to_string()),
            headers: vec![],
            body: Some(e.to_string()),
        }
    }

    fn write_to(&self, stream: &mut TcpStream) {
        let binding = self.as_string();
        let bytes = binding.as_bytes();

        stream.write_all(bytes).expect("Failed")
    }
}
