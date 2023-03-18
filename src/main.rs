use std::{
    env,
    sync::{Arc, RwLock},
};

use web_command::config::Config;

fn main() {
    let c = Config::read_from_config(
        env::var("WEBCOMMAND_CONFIG")
            .unwrap_or("./sites.toml".to_owned())
            .as_str(),
    );

    if let Err(e) = web_command::run(Arc::new(RwLock::new(c.unwrap()))) {
        println!("there was an error:\n{}", e);
    }
}
