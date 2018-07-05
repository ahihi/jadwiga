use ::rocket::Route;
use ::rocket::request::State;
use ::rocket::response::status::NotFound;
use ::rocket_contrib::Json;
use ::webfinger::{Link, Webfinger};

use config::Config;

#[derive(Debug, FromForm)]
struct Query {
    pub resource: String
}

#[get("/.well-known/webfinger?<query>")]
fn find(query: Query, config: State<Config>) -> Result<Json<Webfinger>, NotFound<String>>{
    let parts = query.resource.splitn(2, '@').collect::<Vec<_>>();

    if parts.len() != 2 {
        return Err(NotFound("nyoro~n".to_owned()))
    }
    
    let username = parts[0];
    let host = parts[1];

    if host != config.host || username != config.actor_username {
        return Err(NotFound("nyoro~n".to_owned()))
    }
    
    Ok(Json(Webfinger {
        subject: format!("acct:{}@{}", config.actor_username, config.host),
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
