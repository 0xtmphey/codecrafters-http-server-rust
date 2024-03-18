// Uncomment this block to pass the first stage
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

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

fn process(mut stream: TcpStream, dir: Option<String>) {
    println!("accepted new connection");

    let buffer = BufReader::new(&stream);

    let response: Vec<_> = buffer
        .lines()
        .map(|s| s.unwrap())
        .take_while(|s| !s.is_empty())
        .take(3)
        .collect();

    let path = response[0].split(' ').nth(1);

    match path {
        Some(s) if s.starts_with("/files/") => {
            let filename = &s[7..];
            let file_res = read_file(dir, filename);

            match file_res {
                Some(content) => {
                    let content_type = String::from("Content-type: application/octet-stream");
                    let content_len = format!("Content-length: {}", content.len());

                    Response::ok(vec![content_type, content_len], Some(content))
                        .write_to(&mut stream);
                }
                None => Response::not_found().write_to(&mut stream),
            }
        }
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
        Some(s) if s.starts_with("/user-agent") => {
            let user_agent_res = response.iter().find(|s| s.starts_with("User-Agent:"));

            let user_agent = match user_agent_res {
                Some(value) => value.split(": ").nth(1).unwrap_or(""),
                None => "",
            };

            let content_type = String::from("Content-type: text/plain");
            let content_len_header = format!("Content-length: {}", user_agent.len());
            Response::ok(
                vec![content_type, content_len_header],
                Some(user_agent.to_owned()),
            )
            .write_to(&mut stream);
        }
        Some("/") => Response::ok(vec![], None).write_to(&mut stream),
        _ => Response::not_found().write_to(&mut stream),
    };
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
