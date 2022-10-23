use std::fs;

use serde_derive::Deserialize;
use urlencoding::encode;

#[derive(Debug)]
struct SearchConfig {
    key: String,
    url: String,
    key_with_space: String,
    index_of_replace: usize,
}

#[derive(Debug, Deserialize)]
struct ConfigRedirect {
    key: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    sites: Vec<ConfigRedirect>,
    prefix: char,
}

#[derive(Debug)]
pub struct Config {
    sites: Vec<SearchConfig>,
}

impl Config {
    pub fn read_from_config(path: &str) -> Config {
        let content = fs::read_to_string(path)
            .expect(format!("please provide a config file at {}", path).as_str());

        let raw: RawConfig = toml::from_str(content.as_str())
            .expect(format!("error in the config file at {}", path).as_str());

        Config {
            sites: raw
                .sites
                .into_iter()
                .map(|e| SearchConfig {
                    key: format!("{}{}", raw.prefix, e.key),
                    key_with_space: format!("{}{} ", raw.prefix, e.key),
                    url: e.url.replace("{{s}}", ""),
                    index_of_replace: e
                        .url
                        .find("{{s}}")
                        .expect("please indicate the position to be replaced in an url with {{s}}"),
                })
                .collect(),
        }
    }

    pub fn find_redirect(&self, search_string: &str) -> Option<String> {
        self.sites
            .iter()
            .find(|e| search_string.ends_with(&e.key) || search_string.contains(&e.key_with_space))
            .or_else(|| self.sites.get(0))
            .map(|e| {
                let mut redirect = e.url.clone();
                redirect.insert_str(
                    e.index_of_replace,
                    encode(
                        &search_string
                            .replace(&e.key_with_space, "")
                            .replace(&e.key, ""),
                    )
                    .into_owned()
                    .as_str(),
                );
                redirect
            })
    }
}
