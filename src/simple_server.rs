use std::{
    error::Error,
    io::{BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::anyhow;
use http_bytes::{
    http::{Method, Response},
    parse_request_header_easy, write_response_header, Request,
};

use crate::config::Config;

pub type SResponse = anyhow::Result<Response<Vec<u8>>, http_bytes::http::Error>;

#[derive(Copy, Clone)]
pub enum RequestHandlerFunc {
    ReadFunc(fn(&Request, &Config) -> anyhow::Result<SResponse>),
    WriteFunc(fn(&Request, &mut Config) -> anyhow::Result<SResponse>),
}

struct RequestHandler {
    method: Method,
    path: String,
    f: RequestHandlerFunc,
}

pub struct SimpleServer {
    handlers: Vec<RequestHandler>,
    listener: TcpListener,
    config: Arc<RwLock<Config>>,
}

impl SimpleServer {
    pub fn new(listener: TcpListener, config: Arc<RwLock<Config>>) -> SimpleServer {
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
            let mut s = stream?;
            if let Err(e) = self.handle_client(&mut s) {
                send_error(&mut s, &e);
            }
        }
        Ok(())
    }

    fn handle_client(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let mut buf = vec![0u8; 1024 * 2];

        stream.set_read_timeout(Some(Duration::from_millis(1000)))?;

        stream.set_write_timeout(Some(Duration::from_millis(1000)))?;

        stream.read(&mut buf)?;

        // TODO: If the buffer has not enough data yet, we should read again.
        // In this case the parse_request method returns a Ok(None).
        let request = parse_request_header_easy(&buf)?;

        if let Some((req, _)) = request {
            if let Some(handler) = self
                .handlers
                .iter()
                .find(|h| h.method == req.method() && req.uri().to_string().starts_with(&h.path))
            {
                // TODO: This is too much cloning!
                let h = handler.f.clone();
                let c = Arc::clone(&self.config);
                let mut s = stream.try_clone()?;

                thread::spawn(move || {
                    if let Err(e) = send_response(&mut s, h, &req, c) {
                        if let Err(err) = Response::builder()
                            .status(500)
                            .body(e.to_string().into_bytes())
                            .map(|r| write_response_header(&r, s))
                        {
                            println!("Err {}", err);
                        }
                    }
                });
                return Ok(());
            } else {
                return Err(anyhow!(
                    "Not found for method {} on {}",
                    req.method(),
                    req.uri().to_string()
                ));
            }
        }

        Err(anyhow!("Error while parsing request"))
    }
}

fn send_error(stream: &mut TcpStream, err: &anyhow::Error) {
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
    if let Err(e) = stream.write_all(&response.body()) {
        println!("Err {}", e);
    }
}

fn send_response(
    stream: &mut TcpStream,
    handler: RequestHandlerFunc,
    req: &Request,
    config: Arc<RwLock<Config>>,
) -> anyhow::Result<()> {
    let writer = BufWriter::new(&*stream);
    let handler_res = match handler {
        RequestHandlerFunc::WriteFunc(f) => {
            let mut c = config
                .write()
                .map_err(|e| anyhow!("Err {}", e.to_string()))?;
            (f)(req, &mut c)
        }
        RequestHandlerFunc::ReadFunc(f) => {
            let c = config
                .read()
                .map_err(|e| anyhow!("Err {}", e.to_string()))?;
            (f)(req, &c)
        }
    };
    if let Ok(response) = handler_res? {
        write_response_header(&response, writer)?;
        stream.write_all(response.body())?;
    } else {
        return Err(anyhow!("Error while handling request"));
    }
    Ok(())
}
