// Uncomment this block to pass the first stage
use std::{
    fs,
    net::{TcpListener, TcpStream},
    thread,
};

mod request;
mod errors;
mod header;
mod response;
mod utils;

use response::Response;
use request::{HttpMethod, Request};
use utils::{extract_directory, read_file};

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

fn process(mut stream: TcpStream, dir: Option<String>) {
    let request = Request::try_read(&mut stream);

    let response = match request {
        Ok(req) => handle_request(req, dir),
        Err(e) => Response::error(e),
    };

    response.write_to(&mut stream);
}

fn handle_request(req: Request, dir: Option<String>) -> Response {
    let path = if req.path.ends_with("/") && req.path.len() > 1 {
        req.path.strip_suffix("/").unwrap()
    } else {
        &req.path
    };
    let method = &req.method;

    match (method, path) {
        (HttpMethod::GET, "/") => Response::empty_ok(),

        (HttpMethod::GET, "/user-agent") => {
            let user_agent = &req.headers.iter()
                .find(|header| header.name.to_lowercase() == "user-agent")
                .map_or("None".to_string(), |header| header.value.to_owned());
            let headers = vec![
                format!("Content-Length: {}", user_agent.len()),
                format!("Content-Type: {}", "text/plain"),
            ];
            Response::ok(headers, Some(user_agent.to_owned()))
        }

        (HttpMethod::GET, path) if path.starts_with("/echo/") => {
            let echo = &path[6..];
            let headers = vec![
                format!("Content-Length: {}", echo.len()),
                format!("Content-Type: {}", "text/plain"),
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
                        format!("Content-Type: {}", "application/octet-stream"),
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
    }
}