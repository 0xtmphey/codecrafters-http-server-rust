// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let buffer = BufReader::new(&stream);

                let response: Vec<_> = buffer
                    .lines()
                    .map(|s| s.unwrap())
                    .take_while(|s| !s.is_empty())
                    .take(1)
                    .collect();

                let path = response[0].split(' ').nth(1);

                match path {
                    Some(s) if s.starts_with("/echo/") => {
                        let echo = &s[6..];
                        let content_type = String::from("Content-type: text/plain");
                        let content_len_header = format!("Content-length: {}", echo.len());
                        Response::ok(
                            vec![content_type, content_len_header],
                            Some(echo.to_owned()),
                        )
                        .write_to(&mut stream);
                    }
                    Some("/") => Response::ok(vec![], None).write_to(&mut stream),
                    _ => Response::not_found().write_to(&mut stream),
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
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

    fn not_found() -> Response {
        Response {
            status_code: (404, String::from("Not found")),
            headers: vec![],
            body: None,
        }
    }

    fn write_to(&self, stream: &mut TcpStream) {
        let binding = self.as_string();
        let bytes = binding.as_bytes();

        stream.write_all(bytes).expect("Failed")
    }
}
