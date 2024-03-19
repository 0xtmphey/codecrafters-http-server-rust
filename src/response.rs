use std::io::Write;
use std::net::TcpStream;

#[derive(Debug)]
pub struct Response {
    pub status_code: (usize, String),
    pub headers: Vec<String>,
    pub body: Option<String>,
}

impl Response {
    pub fn ok(headers: Vec<String>, body: Option<String>) -> Response {
        Response {
            status_code: (200, String::from("Ok")),
            headers,
            body,
        }
    }

    pub fn empty_ok() -> Response {
        Response {
            status_code: (200, String::from("OK")),
            headers: vec![],
            body: None,
        }
    }

    pub fn created() -> Self {
        Response {
            status_code: (201, String::from("Created")),
            headers: vec![],
            body: None,
        }
    }

    pub fn not_found() -> Response {
        Response {
            status_code: (404, String::from("Not found")),
            headers: vec![],
            body: None,
        }
    }

    pub fn error(e: anyhow::Error) -> Self {
        Response {
            status_code: (500, "ERROR".to_string()),
            headers: vec![],
            body: Some(e.to_string()),
        }
    }

    pub fn write_to(&self, stream: &mut TcpStream) {
        let binding: String = self.into();
        let bytes = binding.as_bytes();

        stream.write_all(bytes).expect("Failed")
    }
}

impl Into<String> for &Response {
    fn into(self) -> String {
        let headers_str = self.headers.join("\r\n");
        let status_str = format!("HTTP/1.1 {} {}", self.status_code.0, self.status_code.1);
        let body_str = match &self.body {
            Some(b) => b.as_str(),
            None => "",
        };

        format!("{}\r\n{}\r\n\r\n{}", status_str, headers_str, body_str)
    }
}