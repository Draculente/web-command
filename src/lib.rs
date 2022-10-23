use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

pub mod config;
use config::Config;
use thhp::{Request, Status::Complete};
use urlencoding::decode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(config: Arc<Config>) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", env::var("WEBCOMMAND_PORT")?))?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        let c = Arc::clone(&config);
        let mut s = stream?;
        thread::spawn(move || {
            if let Err(e) = handle_client(&mut s, &c) {
                println!("Err {}", e);
                let _ = s.write_all(
                    b"HTTP/1.1 400 Invalid Request\r\n\
                                      Content-Type: text/plain\r\n\
                                      \r\n",
                );
                let _ = s.write_all(e.to_string().as_bytes());
                let _ = s.write_all(b"\n");
            }
        });
    }
    Ok(())
}

//https://github.com/vi/http-bytes/blob/master/examples/http_proxy.rs

fn handle_client(stream: &mut TcpStream, config: &Config) -> Result<()> {
    let mut buf = vec![0u8; 1024 * 2];
    let mut fill_meter = 0usize;

    if let Err(_) = stream.set_read_timeout(Some(Duration::from_millis(1000))) {
        //TODO: Errorhandling;
    }

    if let Err(_) = stream.set_write_timeout(Some(Duration::from_millis(1000))) {
        //TODO: Errorhandling;
    }

    loop {
        fill_meter = fill_meter + stream.read(&mut buf[fill_meter..])?;

        {
            let mut headers = Vec::<thhp::HeaderField>::with_capacity(16);

            match Request::parse(&buf[0..fill_meter], &mut headers) {
                Ok(Complete((ref req, _))) => return handle_request(stream, req, config),
                Ok(thhp::Incomplete) => (),
                Err(_) => todo!(),
            }
        }

        if fill_meter > 1024 * 100 {
            Err("request to big")?;
        }

        buf.resize(fill_meter + 1024, 0u8);

        thread::sleep(Duration::from_millis(1))
    }
}

fn handle_request(stream: &mut TcpStream, req: &Request, config: &Config) -> Result<()> {
    if req.method != "GET" {
        Err("method not supported")?
    };

    // TODO: Better error handling when no prefix is present
    let redirect = decode(req.target.strip_prefix("/").unwrap_or_default())?;

    match config.find_redirect(&redirect.to_owned()) {
        Some(r) => {
            let _ = stream.write_all(
                format!(
                    "HTTP/1.1 301 Moved Permanently\r\n\
                                      Location: {}\r\n\
                                      \n",
                    r
                )
                .as_bytes(),
            );
        }
        None => Err("no redirect specified")?,
    };

    Ok(())
}
