use ::url::{self, Url};

#[derive(Debug)]
pub struct RawConfig {
    pub db_url: String,
    pub root_url: String,
    pub ap_username: String
}

impl RawConfig {
    pub fn validate(self) -> Result<Config, url::ParseError> {
        Ok(Config {
            db_url: self.db_url,
            root_url: Url::parse(&self.root_url)?,
            ap_username: self.ap_username
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub root_url: Url,
    pub ap_username: String
}

impl Config {
    pub fn profile_url(&self) -> &str {
        self.root_url.as_str()
    }    
}
