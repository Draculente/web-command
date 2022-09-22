use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    pub key: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Config(pub Vec<SearchConfig>);

impl Config {
    pub fn read_from_config(path: &str) -> Config {
        let content = fs::read_to_string(path)
            .expect(format!("please provide a config file at {}", path).as_str());

        //toml::from_str(content.as_str()).expect(format!("error in the config file at {}", path).as_str())
        serde_json::from_str(content.as_str())
            .expect(format!("error in the config file at {}", path).as_str())
    }
}
