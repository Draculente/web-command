use std::{
    error::Error,
    io::{BufWriter, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::anyhow;
use http_bytes::{http::Response, write_response_header};

use crate::{
    config::Config,
    http::{HttpMethod, HttpRequest},
};

pub type SResponse = anyhow::Result<Response<Vec<u8>>, http_bytes::http::Error>;

#[derive(Copy, Clone)]
pub enum RequestHandlerFunc {
    ReadFunc(fn(&HttpRequest, &Config) -> anyhow::Result<SResponse>),
    WriteFunc(fn(&HttpRequest, &mut Config) -> anyhow::Result<SResponse>),
}

struct RequestHandler {
    method: HttpMethod,
    path: String,
    f: RequestHandlerFunc,
}

impl RequestHandler {
    fn matches_on(&self, request: &HttpRequest) -> bool {
        self.method == request.get_method() && request.get_uri().to_string().starts_with(&self.path)
    }
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

    pub fn add_handler(&mut self, method: HttpMethod, path: &str, f: RequestHandlerFunc) {
        self.handlers.push(RequestHandler {
            method,
            path: path.to_string(),
            f,
        });
    }

    pub fn run(&self) -> std::result::Result<(), Box<dyn Error>> {
        let addr = self
            .listener
            .local_addr()
            .expect("Listener must have an addr");
        println!("Server listening on {}:{}", addr.ip(), addr.port());
        for stream in self.listener.incoming() {
            let mut s = stream?;
            if let Err(e) = self.handle_client(&mut s) {
                send_error(&mut s, &e);
            }
        }
        Ok(())
    }

    fn handle_client(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        stream.set_read_timeout(Some(Duration::from_millis(1000)))?;

        stream.set_write_timeout(Some(Duration::from_millis(1000)))?;

        let req = HttpRequest::from_stream(stream).map_err(|e| anyhow!(e))?;

        if let Some(handler) = self.handlers.iter().find(|h| h.matches_on(&req)) {
            // TODO: This is too much cloning!
            let h = handler.f.clone();

            let c = Arc::clone(&self.config);
            let mut s = stream.try_clone()?;

            thread::spawn(move || {
                if let Err(e) = send_response(&mut s, h, &req, c) {
                    eprintln!("Error sending response: {}", e);
                    if let Err(err) = Response::builder()
                        .status(500)
                        .body(e.to_string().into_bytes())
                        .map(|r| write_response_header(&r, s))
                    {
                        eprintln!("Err {}", err);
                    }
                }
            });
            return Ok(());
        } else {
            return Err(anyhow!(
                "Not found for method {} on {}",
                req.get_method(),
                req.get_uri().to_string()
            ));
        }
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
        eprintln!("Err {}", e);
    }
    if let Err(e) = stream.write_all(&response.body()) {
        eprintln!("Err {}", e);
    }
}

fn send_response(
    stream: &mut TcpStream,
    handler: RequestHandlerFunc,
    req: &HttpRequest,
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
        stream.flush()?;
    } else {
        return Err(anyhow!("Error while handling request"));
    }
    Ok(())
}
