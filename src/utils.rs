use std::{env, fs};
use anyhow::anyhow;

pub fn extract_directory() -> Option<String> {
    for (i, arg) in env::args().enumerate() {
        if arg == "--directory" {
            return env::args().nth(i + 1);
        }
    }

    None
}

pub fn read_file(dir: Option<String>, filename: &str) -> Option<String> {
    let path = concat_path(dir, filename);

    match path {
        Some(p) => fs::read_to_string(p).ok(),
        None => None,
    }
}

fn concat_path(dir: Option<String>, file: &str) -> Option<String> {
    dir.map(|d| {
        let delimiter = if d.ends_with('/') { "" } else { "/" };
        format!("{}{}{}", d, delimiter, file)
    })
}