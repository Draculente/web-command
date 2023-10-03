use anyhow::anyhow;
use std::{env, fs};

use serde_derive::Deserialize;
use urlencoding::encode;

#[derive(Debug)]
struct Replaceable {
    key: String,
    url: String,
    key_with_space: String,
    index_of_replace: usize,
}

#[derive(Debug, Deserialize)]
struct ConfigRedirect {
    key: String,
    url: String,
    alias: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    sites: Vec<ConfigRedirect>,
    prefix: char,
}

#[derive(Debug)]
pub struct Config {
    pub location: String,
    sites: Vec<Replaceable>,
    pub is_config_host: bool,
}

impl Replaceable {
    fn new(prefix: char, key: String, url: String) -> Replaceable {
        let key_with_space = format!("{}{} ", prefix, key);
        let index_of_replace = url
            .find("{{s}}")
            .expect("please indicate the position to be replaced in an url with {{s}}");
        let url = url.replace("{{s}}", "");
        let key = format!("{}{}", prefix, key);
        Replaceable {
            key,
            url,
            key_with_space,
            index_of_replace,
        }
    }
}

impl Config {
    pub fn read_from_config(location: &str) -> anyhow::Result<Config> {
        let is_config_host = env::var("WEBCOMMAND_HOST_MODE") == Ok("true".to_owned());
        let content = load_config(location, is_config_host);

        let raw: RawConfig = toml::from_str(content?.as_str())?;

        let sites: Vec<Replaceable> = raw
            .sites
            .into_iter()
            .flat_map(|e| {
                let mut aliases = e.alias.unwrap_or_default();
                aliases.push(e.key.clone());
                aliases
                    .into_iter()
                    .map(move |alias| Replaceable::new(raw.prefix, alias, e.url.clone()))
            })
            .collect();

        Ok(Config {
            location: location.to_string(),
            is_config_host,
            sites,
        })
    }

    pub fn reload_config(&mut self) -> anyhow::Result<()> {
        let c = Config::read_from_config(&self.location)?;
        self.sites = c.sites;
        Ok(())
    }

    pub fn find_redirect(&self, search_string: &str) -> String {
        let e = self
            .sites
            .iter()
            .map(|e| (e, true))
            .find(|e| {
                search_string.ends_with(&e.0.key) || search_string.contains(&e.0.key_with_space)
            })
            .or_else(|| self.sites.get(0).map(|e| (e, false)))
            .expect("no commands found");

        let mut redirect_url = e.0.url.clone();
        // let search_string = search_string.replace(&e.0.key_with_space, "");
        let search_string = if e.1 {
            // Redirect was found by key, so we can remove the key from the search string
            search_string
                .replace(&e.0.key_with_space, "")
                .replace(&e.0.key, "")
        } else {
            // No redirect was found. We redirect to the default site and do not replace anything
            search_string.to_owned()
        };

        // We insert the search string at the position of the {{s}} placeholder
        redirect_url.insert_str(
            e.0.index_of_replace,
            encode(&search_string).into_owned().as_str(),
        );
        redirect_url
    }
}

fn load_config(location: &str, is_config_host: bool) -> anyhow::Result<String> {
    println!(
        "Executing as {}.",
        if is_config_host {
            "config host"
        } else {
            "config mirror"
        }
    );

    if is_config_host {
        fs::read_to_string(location).map_err(|_| anyhow!("please provide the config file"))
    } else {
        let config_host = env::var("WEBCOMMAND_CONFIG").map_err(|_| {
            anyhow!("please provide the url to the config host in WEBCOMMAND_CONFIG.")
        })?;
        let config_host = get_config_url(&config_host);
        let config = reqwest::blocking::get(config_host)?.text()?;
        Ok(config)
    }
}

fn append_to_url(host: &str, path: &str) -> String {
    let mut config_host = host.strip_suffix("/").unwrap_or_else(|| &host).to_owned();
    config_host.push_str(path);
    config_host
}

pub fn get_config_url(host: &str) -> String {
    append_to_url(&host, "/u/")
}

pub fn get_reload_url(host: &str) -> String {
    append_to_url(&host, "/r/")
}
