use ::url::{self, Url};

#[derive(Debug)]
pub struct RawConfig {
    pub db_url: String,
    pub pub_key: String,
    pub root_url: String,
    pub actor_username: String,
    pub actor_name: String,
    pub media_dir: String
}

impl RawConfig {
    pub fn validate(self) -> Result<Config, url::ParseError> {
        Ok(Config {
            db_url: self.db_url,
            pub_key: self.pub_key,
            root_url: Url::parse(&self.root_url)?,
            actor_username: self.actor_username,
            actor_name: self.actor_name,
            media_dir: self.media_dir
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub pub_key: String,
    pub root_url: Url,
    pub actor_username: String,
    pub actor_name: String,
    pub media_dir: String
}

impl Config {
    pub fn actor_url(&self) -> String {
        self.root_url.as_str().to_owned()
    }

    pub fn inbox_url(&self) -> String {
        self.root_url.join("/_inbox").unwrap().as_str().to_owned()
    }
    
    pub fn outbox_url(&self) -> String {
        self.root_url.join("/_outbox").unwrap().as_str().to_owned()
    }

    pub fn post_url(&self, uri_name: &str) -> String {
        self.root_url.join(&format!("/{}", uri_name)).unwrap().as_str().to_owned()
    }

    pub fn activity_url(&self, uri_name: &str) -> String {
        self.root_url.join(&format!("/_activity/{}", uri_name)).unwrap().as_str().to_owned()
    }

    pub fn media_url(&self, file: &str) -> String {
        self.root_url.join(&format!("/_media/{}", file)).unwrap().as_str().to_owned()
    }
}
