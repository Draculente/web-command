use std::{
    error::Error,
    fmt,
    io::{BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

use http_bytes::{
    http::{Method, Response},
    parse_request_header_easy, write_response_header, Request,
};

use crate::config::Config;

pub type Result<T> = std::result::Result<T, SimpleServerError>;

pub type SResponse = std::result::Result<Response<Vec<u8>>, http_bytes::http::Error>;

#[derive(Debug)]
pub enum SimpleServerError {
    ParsingRequest,
    HandlingClient,
    HandlingRequest,
    NotFound,
    NoRedirect,
}

impl Error for SimpleServerError {}

impl From<std::io::Error> for SimpleServerError {
    fn from(_: std::io::Error) -> Self {
        SimpleServerError::HandlingClient
    }
}

impl fmt::Display for SimpleServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimpleServerError::ParsingRequest => write!(f, "Error while parsing request"),
            SimpleServerError::HandlingClient => write!(f, "Error while handling client"),
            SimpleServerError::HandlingRequest => write!(f, "Error while handling request"),
            SimpleServerError::NotFound => write!(f, "Not found"),
            SimpleServerError::NoRedirect => write!(f, "No redirect"),
        }
    }
}

pub type RequestHandlerFunc = fn(&Request, &Config) -> Result<SResponse>;

struct RequestHandler {
    method: Method,
    path: String,
    f: RequestHandlerFunc,
}

pub struct SimpleServer {
    handlers: Vec<RequestHandler>,
    listener: TcpListener,
    config: Arc<Config>,
}

impl SimpleServer {
    pub fn new(listener: TcpListener, config: Arc<Config>) -> SimpleServer {
        SimpleServer {
            handlers: Vec::new(),
            listener,
            config,
        }
    }

    pub fn add_handler(&mut self, method: Method, path: &str, f: RequestHandlerFunc) {
        self.handlers.push(RequestHandler {
            method,
            path: path.to_string(),
            f,
        });
    }

    pub fn run(&self) -> std::result::Result<(), Box<dyn Error>> {
        for stream in self.listener.incoming() {
            let s = stream?;
            if let Err(e) = self.handle_client(s) {
                println!("Err {}", e);
            }
        }
        Ok(())
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<()> {
        let mut buf = vec![0u8; 1024 * 2];

        if let Err(_) = stream.set_read_timeout(Some(Duration::from_millis(1000))) {
            //TODO: Errorhandling;
        }

        if let Err(_) = stream.set_write_timeout(Some(Duration::from_millis(1000))) {
            //TODO: Errorhandling;
        }

        stream.read(&mut buf)?;

        if let Ok(Some((req, _))) = parse_request_header_easy(&buf) {
            if let Some(handler) = self
                .handlers
                .iter()
                .find(|h| h.method == req.method() && req.uri().to_string().starts_with(&h.path))
            {
                let h = handler.f.clone();
                let c = Arc::clone(&self.config);
                thread::spawn(move || {
                    if let Err(e) = send_response(&mut stream, h, &req, &c) {
                        if let Err(err) = Response::builder()
                            .status(500)
                            .body(e.to_string().into_bytes())
                            .map(|r| write_response_header(&r, stream))
                        {
                            println!("Err {}", err);
                        }
                    }
                });
                return Ok(());
            } else {
                send_error(&mut stream, SimpleServerError::NotFound);
            }
        }

        Err(SimpleServerError::ParsingRequest)
    }
}

fn send_error(stream: &mut TcpStream, err: SimpleServerError) {
    println!("Err {}", err);
    let message = err.to_string().into_bytes();
    let writer = BufWriter::new(&*stream);
    let response = Response::builder()
        .status(500)
        .header("content-length", message.len())
        .body(&message)
        .unwrap();
    if let Err(e) = write_response_header(&response, writer) {
        println!("Err {}", e);
    }
    if let Err(e) = stream.write_all(&message) {
        println!("Err {}", e);
    }
}

fn send_response(
    stream: &mut TcpStream,
    handler: RequestHandlerFunc,
    req: &Request,
    config: &Config,
) -> Result<()> {
    let writer = BufWriter::new(&*stream);
    if let Ok(response) = (handler)(req, config)? {
        write_response_header(&response, writer)?;
        stream.write_all(response.body())?;
    } else {
        return Err(SimpleServerError::HandlingRequest);
    }
    Ok(())
}
