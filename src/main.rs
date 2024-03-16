// Uncomment this block to pass the first stage
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
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
                    Some("/") => stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").expect("Error"),
                    _ => stream
                        .write_all(b"HTTP/1.1 404 NOT_FOUND\r\n\r\n")
                        .expect("Error"),
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
