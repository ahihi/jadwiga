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
    let acct = format!("acct:{}@{}", config.actor_username, config.host);
    
    if query.resource != acct {
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
