use ::rocket::Route;
use ::rocket::request::State;
use ::rocket_contrib::Json;
use ::webfinger::{Link, Webfinger};

use api::error::Error;
use config::Config;

#[derive(Debug, FromForm)]
struct Query {
    pub resource: String
}

#[get("/.well-known/webfinger?<query>")]
fn find(query: Query, config: State<Config>) -> Result<Json<Webfinger>, Error>{
    let host = config.root_url.host_str()
        .unwrap_or("");

    let acct = format!("acct:{}@{}", config.actor_username, host);
    
    if query.resource != acct {
        return Err(Error::NotFound)
    }
    
    Ok(Json(Webfinger {
        subject: acct,
        aliases: vec![config.actor_url()],
        links: vec![
            Link {
                rel: "self".to_owned(),
                mime_type: Some("application/activity+json".to_owned()),
                href: config.actor_url()
            }
        ]
    }))
}

pub fn routes() -> Vec<Route> {
    routes![find]
}
