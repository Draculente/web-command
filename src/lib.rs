use std::{
    env, fs,
    net::TcpListener,
    sync::{Arc, RwLock},
};

pub mod config;
mod simple_server;
use clap::crate_version;
use config::{get_config_url, get_reload_url, Config};
use http_bytes::{
    http::{Method, Response, StatusCode},
    Request,
};
use simple_server::{RequestHandlerFunc, SResponse, SimpleServer};
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
        "/i/",
        RequestHandlerFunc::ReadFunc(|_, _| {
            let info = format!("WSH v{}\nMade with love in the European Union\nhttps://github.com/Draculente/web-command\n", crate_version!());
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .header("Content-Length", info.len())
                .body(info.as_bytes().to_vec()))
        }),
    );
    server.add_handler(
        Method::GET,
        "/",
        RequestHandlerFunc::ReadFunc(redirect_handler),
    );
    server.run()?;

    Ok(())
}

fn send_config_file(_: &Request, config: &Config) -> anyhow::Result<SResponse> {
    if config.is_config_host {
        let response = fs::read_to_string(config.location.as_str())?;
        let res_bytes = response.as_bytes();

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .header("Access-Control-Allow-Origin", "*")
            .header("Content-Length", res_bytes.len())
            .body(res_bytes.to_vec()))
    } else {
        Ok(Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header("Location", get_config_url(&config.location))
            .body(vec![]))
    }
}

fn reload_config_handler(_: &Request, config: &mut Config) -> anyhow::Result<SResponse> {
    if !config.is_config_host {
        if let Err(e) = reqwest::blocking::get(get_reload_url(&config.location)) {
            eprintln!("Error while triggering reload on config host: {}", e);
        }
    }

    config.reload_config()?;
    let response_text = "Reloaded configuration".as_bytes();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .header("Content-Length", response_text.len())
        .body(response_text.to_vec());
    Ok(response)
}

fn redirect_handler(req: &Request, config: &Config) -> anyhow::Result<SResponse> {
    let raw_search_string = req
        .uri()
        .to_string()
        .strip_prefix("/")
        .map(|s| s.replace("+", " "))
        .unwrap_or("".to_owned());

    let search_string = decode(&raw_search_string)?;

    let redirect_url = config.find_redirect(&search_string.to_owned());
    Ok(Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header("Location", redirect_url)
        .body(vec![]))
}
