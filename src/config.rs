use ::url::{self, Url};

#[derive(Debug)]
pub struct RawConfig {
    pub db_url: String,
    pub root_url: String,
    pub actor_preferred_username: String,
    pub actor_name: String
}

impl RawConfig {
    pub fn validate(self) -> Result<Config, url::ParseError> {
        Ok(Config {
            db_url: self.db_url,
            root_url: Url::parse(&self.root_url)?,
            actor_preferred_username: self.actor_preferred_username,
            actor_name: self.actor_name
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub root_url: Url,
    pub actor_preferred_username: String,
    pub actor_name: String
}

impl Config {
    pub fn actor_url(&self) -> String {
        self.root_url.as_str().to_owned()
    }

    pub fn outbox_url(&self) -> String {
        self.root_url.join("/_outbox").unwrap().as_str().to_owned()
    }
}
