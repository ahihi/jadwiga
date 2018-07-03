use ::rocket::http::Status;
use ::rocket::request::{self, FromRequest};
use ::rocket::{Request, State, Outcome};

pub struct Config {
    pub domain: String,
    pub ap_user_name: String
}
