#[macro_use]
extern crate rocket;

mod config;

use std::{env, process};

use config::Config;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::{http::uri::Absolute, response::Redirect, State};
use urlencoding::{decode, encode};

struct SearchString(String);

#[rocket::async_trait]
impl<'r, 'a> FromRequest<'r> for SearchString {
    type Error = std::fmt::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.uri().path().raw_segments().nth(0) {
            Some(param) => Outcome::Success(SearchString(
                decode(&param.as_str().replace("+", "%20"))
                    .unwrap()
                    .into_owned(),
            )),
            None => Outcome::Success(SearchString("".to_string())),
        }
    }
}

impl SearchString {
    fn matches(&self, key: &str) -> bool {
        self.0.ends_with(key) || self.0.contains(&format!("{} ", key))
    }

    fn encode(self, key: &str) -> String {
        encode(&self.0.replace(&format!("{} ", key), "").replace(&key, "")).into_owned()
    }
}

#[get("/<_>")]
fn index(string: SearchString, config: &State<Config>) -> Redirect {
    let sites = &config.sites;
    let url = sites
        .iter()
        .find(|e| string.matches(e.key.as_str()))
        .or_else(|| sites.get(0))
        .map(|e| {
            e.url
                .clone()
                .replace("{{s}}", string.encode(e.key.as_str()).as_str())
        })
        .unwrap();

    Redirect::to(Absolute::parse_owned(url).unwrap())
}

#[catch(default)]
fn not_found(_: Status, _: &Request) -> &'static str {
    "Sorry. There was an error."
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Config::read_from_config(
            &env::args().nth(1).unwrap_or("./sites.toml".to_string()),
        ))
        .mount("/", routes![index])
        .register("/", catchers![not_found])
}
