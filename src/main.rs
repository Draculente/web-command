#[macro_use]
extern crate rocket;

mod config;

use config::Config;
use rocket::http::Status;
use rocket::Request;
use rocket::{http::uri::Absolute, response::Redirect, State};
use urlencoding::encode;

#[get("/<search_string>")]
fn index(search_string: &str, config: &State<Config>) -> Redirect {
    let url = config
        .inner()
        .0
        .iter()
        .find(|e| search_string.contains(e.key.as_str()))
        .or_else(|| config.0.get(0))
        .map(|e| {
            e.url.clone().replace(
                "{{s}}",
                encode(&search_string.replace(&e.key, ""))
                    .into_owned()
                    .as_str(),
            )
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
        .manage(Config::read_from_config("./sites.json"))
        .mount("/", routes![index])
        .register("/", catchers![not_found])
}
