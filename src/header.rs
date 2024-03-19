use std::fmt;
use std::fmt::Formatter;
use crate::errors::ParsingError;

#[derive(Debug)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl TryFrom<&str> for HttpHeader {
    type Error = ParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split(':');

        match (parts.next(), parts.next()) {
            (Some(name), Some(val)) => {
                Ok(HttpHeader {
                    name: name.to_string(),
                    value: val.trim().to_string(),
                })
            }
            (_, _) => Err(ParsingError::UnsupportedHeaderFormat(value.to_string()))
        }
    }
}

impl TryFrom<String> for HttpHeader {
    type Error = ParsingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Into<String> for HttpHeader {
    fn into(self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}
impl fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Header({}: {})", self.name, self.value)
    }
}