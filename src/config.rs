use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    pub key: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub sites: Vec<SearchConfig>,
    pub prefix: char,
}

impl Config {
    pub fn read_from_config(path: &str) -> Config {
        let content = fs::read_to_string(path)
            .expect(format!("please provide a config file at {}", path).as_str());

        let mut config: Config = toml::from_str(content.as_str())
            .expect(format!("error in the config file at {}", path).as_str());
        //serde_json::from_str(content.as_str()).expect(format!("error in the config file at {}", path).as_str())

        config.sites = config
            .sites
            .into_iter()
            .map(|mut e| {
                e.key.insert(0, config.prefix);
                e
            })
            .collect();

        config
    }
}
