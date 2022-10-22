use std::{
    env,
    io::Read,
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

mod config;
use config::Config;
use thhp::{Request, Status::Complete};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn run(config: Arc<Config>) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", env::var("WEBCOMMAND_PORT")?))?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        let c = Arc::clone(&config);
        let mut s = stream?;
        thread::spawn(move || {
            handle_client(&mut s, &c);
            //TODO: Send error to client if handle_client errors
        });
    }
    Ok(())
}

//https://github.com/vi/http-bytes/blob/master/examples/http_proxy.rs

fn handle_client(stream: &mut TcpStream, config: &Config) -> Result<()> {
    let mut buf = vec![0u8; 1024];
    let mut fill_meter = 0usize;
    let mut headers = Vec::<thhp::HeaderField>::with_capacity(16);

    if let Err(_) = stream.set_read_timeout(Some(Duration::from_millis(1000))) {
        //TODO: Errorhandling;
    }

    loop {
        let read1 = stream.read(&mut buf[fill_meter..])?;

        {
            let b = &buf[0..(fill_meter + read1)];
            match Request::parse(b, &mut headers) {
                Ok(Complete((ref req, len))) => return handle_request(req, config),
                Ok(thhp::Incomplete) => (),
                Err(_) => todo!(),
            }
        }
    }
}

fn handle_request(req: &Request, config: &Config) -> Result<()> {
    // TODO: logic from old request handler.
    todo!()
}
