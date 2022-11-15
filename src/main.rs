use std::sync::Arc;

use web_command::config::Config;

fn main() {
    let c = Config::read_from_config("./sites.toml");

    if let Err(e) = web_command::run(Arc::new(c)) {
        println!("there was an error:\n{}", e);
    }
}
