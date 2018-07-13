use ::std::io;
use ::std::path::{Path, PathBuf};

use ::chrono::offset::{TimeZone, Utc};
use ::diesel::prelude::*;
use ::rocket::{
    Route,
    request::{
        State
    },
    response::{
        NamedFile
    }
};
use ::rocket_contrib::Json;
use ::serde_json::Value;

use api::error::Error;
use config::Config;
use db::Database;
use models;
use schema;
use sig::ValidSignature;

mod ns {
    pub const ACTIVITYSTREAMS: &str = "https://www.w3.org/ns/activitystreams";
    pub const PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

    pub const SECURITY: &str = "https://w3id.org/security/v1";
}


fn get_actor(config: &Config, _database: &Database) -> Result<Value, Error> {
    let actor_url = config.actor_url();
        
    Ok(json!({
        "@context": [
	    ns::ACTIVITYSTREAMS,
	    ns::SECURITY
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
	},

        "icon": {
            "type": "Image",
            "mediaType": "image/jpeg",
            "url": config.media_url("icon.jpg")
        }
    }))
}

fn format_timestamp(timestamp: i32) -> String {
    Utc.timestamp(timestamp as i64, 0)
        .to_rfc3339()
}

fn get_content(post: &models::Post) -> String {
    let mut content = String::new();
    
    for piece in post.body.pieces.iter() {
        let text: &str = match piece {
            models::Piece::Html(html) => html,
            _ => "FIXME"
        };
        
        content.push_str(&text);
    }

    content
}

fn get_create_note(post: &models::Post, config: &Config) -> Value {
    let actor_url = config.actor_url();
    let published = format_timestamp(post.datetime);
    
    json!({
        "type": "Create",
        "id": config.activity_url(&post.uri_name),
        "actor": actor_url,
        "published": published,
        "to": [
            ns::PUBLIC
        ],
        "object": {
            "type": "Note",
            "id": config.post_url(&post.uri_name),
            "attributedTo": actor_url,
            "published": published,
            "name": post.title,
            "content": get_content(post)
        }
    })
}

fn get_outbox(config: &Config, database: &Database) -> Result<Value, Error> {
    let posts = schema::posts::table
        .order(schema::posts::id.desc())
        .load::<models::Post>(&database.conn)?;
    
    let items = posts.into_iter()
        .map(|post| get_create_note(&post, config))
        .collect::<Vec<_>>();
    
    Ok(json!({
        "@context": ns::ACTIVITYSTREAMS,

        "type": "OrderedCollection",
        "id": config.outbox_url(),
        "totalItems": items.len(),
        "items": items
    }))
}

#[get("/")]
fn actor(config: State<Config>, database: Database) -> Result<Json<Value>, Error> {
    Ok(Json(get_actor(&config, &database)?))
}

#[post("/_inbox")]
fn inbox(config: State<Config>, database: Database, signature: Result<ValidSignature, Error>) -> Result<Json<Value>, Error> {
    let signature = signature?;
    
    Ok(Json(Value::Null))
}

#[get("/_outbox")]
fn outbox(config: State<Config>, database: Database) -> Result<Json<Value>, Error> {
    Ok(Json(get_outbox(&config, &database)?))
}


#[get("/_media/<file..>")]
fn media(file: PathBuf, config: State<Config>) -> Result<NamedFile, Error> {
    let f = NamedFile::open(Path::new(&config.media_dir).join(file))
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound =>
                Error::NotFound,
            _ =>
                Error::internal(e)
        })?;

    Ok(f)
}

pub fn routes() -> Vec<Route> {
    routes![actor, inbox, outbox, media]
}
