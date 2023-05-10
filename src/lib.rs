use std::{
    env, fs,
    net::TcpListener,
    sync::{Arc, RwLock},
};

pub mod config;
mod simple_server;
use config::{get_config_url, Config};
use http_bytes::{
    http::{Method, Response, StatusCode},
    Request,
};
use simple_server::{
    RequestHandlerFunc, Result as SResult, SResponse, SimpleServer, SimpleServerError,
};
use urlencoding::decode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(config: Arc<RwLock<Config>>) -> Result<()> {
    let listener = TcpListener::bind(format!(
        "0.0.0.0:{}",
        env::var("WEBCOMMAND_PORT").unwrap_or("8012".to_owned())
    ))?;

    let mut server = SimpleServer::new(listener, config);
    server.add_handler(
        Method::GET,
        "/u/",
        RequestHandlerFunc::ReadFunc(send_config_file),
    );
    server.add_handler(
        Method::GET,
        "/r/",
        RequestHandlerFunc::WriteFunc(reload_config_handler),
    );
    server.add_handler(
        Method::GET,
        "/",
        RequestHandlerFunc::ReadFunc(redirect_handler),
    );
    server.run()?;

    Ok(())
}

fn send_config_file(_: &Request, config: &Config) -> SResult<SResponse> {
    if config.is_config_host {
        let response = fs::read_to_string(config.path.as_str())?;
        let res_bytes = response.as_bytes();

        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .header("Access-Control-Allow-Origin", "*")
            .header("Content-Length", res_bytes.len())
            .body(res_bytes.to_vec()))
    } else {
        Ok(Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header("Location", get_config_url(&config.path))
            .body(vec![]))
    }
}

fn reload_config_handler(_: &Request, config: &mut Config) -> SResult<SResponse> {
    if !config.is_config_host {
        if let Err(e) = reqwest::blocking::get(&config.path) {
            eprintln!("Error while triggering reload on config host: {}", e);
        }
    }

    config
        .reload_config()
        .map_err(|_| SimpleServerError::HandlingRequest)?;
    let response_text = "Reloaded configuration".as_bytes();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .header("Content-Length", response_text.len())
        .body(response_text.to_vec());
    Ok(response)
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
