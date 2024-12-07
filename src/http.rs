use std::{
    fmt::Display,
    io::{ErrorKind, Read},
};

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HttpMethod {
    Get,
    Post,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
        };
        f.write_str(string)
    }
}

impl HttpMethod {
    fn from_str(method_str: &str) -> Result<Self, HttpParseError> {
        match method_str {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err(HttpParseError::UnsupportedMethod),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: HttpMethod,
    uri: String,
    version: String,
}

#[derive(Debug)]
pub enum HttpParseError {
    UriTooLong,
    TcpStreamError(ErrorKind),
    NotValidHttp,
    UnsupportedMethod,
}

impl Into<anyhow::Error> for HttpParseError {
    fn into(self) -> anyhow::Error {
        match self {
            HttpParseError::UriTooLong => anyhow!("Uri is too long"),
            HttpParseError::TcpStreamError(error_kind) => anyhow!(error_kind),
            HttpParseError::NotValidHttp => {
                anyhow!("Error parsing the http request - it does not seem to be valid")
            }
            HttpParseError::UnsupportedMethod => anyhow!("Unsupported method"),
        }
    }
}

impl HttpRequest {
    pub fn uri_without_starting_slash(&self) -> &str {
        &self
            .uri
            .strip_prefix("/")
            .expect("The uri MUST have a / prefix")
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_method(&self) -> HttpMethod {
        self.method
    }

    pub fn from_stream(stream: &mut impl Read) -> Result<Self, HttpParseError> {
        let mut buf = vec![0u8; 1024 * 2];
        while !buf.contains(&10) {
            // Check if the buffer is full
            if buf[buf.len() - 1] != 0 {
                eprintln!("Uri is too long");
                return Err(HttpParseError::UriTooLong);
            }

            if let Err(e) = stream.read(&mut buf) {
                eprintln!("Error reading stream: {e}");
                // We can continue to read if the read gets interrupted
                if e.kind() == ErrorKind::Interrupted {
                    continue;
                }
                return Err(HttpParseError::TcpStreamError(e.kind()));
            }
        }
        let first_line: String = buf
            .into_iter()
            .take_while(|d| *d != 10)
            .map(|d| d as char)
            .collect();

        let request = Self::parse_from_string(first_line);

        request
    }

    fn parse_from_string(first_line: String) -> Result<Self, HttpParseError> {
        let parts: Vec<&str> = first_line.trim().split(" ").collect();
        if parts.len() != 3 {
            return Err(HttpParseError::NotValidHttp);
        }

        let method = HttpMethod::from_str(parts[0])?;
        let uri = parts[1].to_string().replace("+", "%20");
        let version = parts[2].to_string();

        if !uri.starts_with("/") {
            return Err(HttpParseError::NotValidHttp);
        }

        Ok(Self {
            method,
            uri,
            version,
        })
    }
}
