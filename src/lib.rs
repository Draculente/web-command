#![feature(try_trait_v2)]
use std::{env, fs, net::TcpListener, sync::Arc};

pub mod config;
mod simple_server;
use config::Config;
use http_bytes::{
    http::{Method, Response, StatusCode},
    Request,
};
use simple_server::{Result as SResult, SResponse, SimpleServer, SimpleServerError};
use urlencoding::decode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(config: Arc<Config>) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", env::var("WEBCOMMAND_PORT")?))?;

    let mut server = SimpleServer::new(listener, config);
    server.add_handler(Method::GET, "/u/", send_config_file);
    server.add_handler(Method::GET, "/", redirect_handler);
    server.run()?;

    Ok(())
}

fn send_config_file(_: &Request, config: &Config) -> SResult<SResponse> {
    let file = fs::read_to_string(config.path.as_str())?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Length", file.as_bytes().len())
        .body(file.as_bytes().to_vec()))
}

fn redirect_handler(req: &Request, config: &Config) -> SResult<SResponse> {
    let raw_redirect = req
        .uri()
        .to_string()
        .strip_prefix("/")
        .map(|s| s.replace("+", " "))
        .unwrap_or("".to_owned());

    match decode(&raw_redirect) {
        Err(_) => Err(SimpleServerError::HandlingRequest)?,
        Ok(redirect) => match config.find_redirect(&redirect.to_owned()) {
            Some(r) => Ok(Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header("Location", r)
                .body(vec![])),
            None => Err(SimpleServerError::NoRedirect)?,
        },
    }
}
