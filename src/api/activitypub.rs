use ::std::path::{Path, PathBuf};

use ::diesel::prelude::*;
use ::failure::Error;
use ::rocket::Route;
use ::rocket::request::State;
use ::rocket::response::{
    NamedFile,
    status::NotFound
};
use ::rocket_contrib::Json;
use ::serde_json::Value;

use config::Config;
use db::Database;
use models;
use schema;

mod context {
    pub const ACTIVITYSTREAMS: &str = "https://www.w3.org/ns/activitystreams";
    pub const SECURITY: &str = "https://w3id.org/security/v1";
}

fn ap_run<T, F>(act: F) -> Result<Json<T>, NotFound<String>>
    where F: Fn() -> Result<T, Error>
{
    act()
        .map(|data| Json(data))
        .map_err(|e| NotFound(format!("{}", e)))
}

fn get_actor(config: &Config, _database: &Database) -> Result<Value, Error> {
    let actor_url = config.actor_url();
        
    Ok(json!({
        "@context": [
	    context::ACTIVITYSTREAMS,
	    context::SECURITY
	],
        
	"type": "Person",
        "id": actor_url,
        "preferredUsername": config.actor_username,
        "name": config.actor_name,
	"inbox": config.inbox_url(),
        "outbox": config.outbox_url(),

	"publicKey": {
	    "id": format!("{}#main-key", actor_url),
	    "owner": actor_url,
	    "publicKeyPem": config.pub_key
	}
    }))
}

fn get_note(post: &models::Post, config: &Config) -> Result<Value, Error> {
    Ok(json!({
        "type": "Create",
        "actor": config.actor_url(),
        "object": config.post_url(&post.uri_name)
    }))
}

fn get_outbox(config: &Config, database: &Database) -> Result<Value, Error> {
    let posts = schema::posts::table
        .load::<models::Post>(&database.conn)?;
    
    let items = posts.iter()
        .map(|post| get_note(post, config))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(json!({
        "@context": context::ACTIVITYSTREAMS,

        "type": "OrderedCollection",
        "id": config.outbox_url(),
        "totalItems": posts.len(),
        "items": items
    }))
}

#[get("/")]
fn actor(config: State<Config>, database: Database) -> Result<Json<Value>, NotFound<String>> {
    ap_run(|| get_actor(&config, &database))
}

#[get("/_outbox")]
fn outbox(config: State<Config>, database: Database) -> Result<Json<Value>, NotFound<String>> {
    ap_run(|| get_outbox(&config, &database))
}

#[get("/_media/<file..>")]
fn media(file: PathBuf, config: State<Config>) -> Result<NamedFile, Error> {
    Ok(NamedFile::open(Path::new(&config.media_dir).join(file))?)
}

pub fn routes() -> Vec<Route> {
    routes![actor, outbox, media]
}
