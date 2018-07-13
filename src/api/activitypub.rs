use ::std::io::{self, Read};
use ::std::path::{Path, PathBuf};

use ::chrono::offset::{TimeZone, Utc};
use ::diesel::prelude::*;
use ::rocket::{
    Data, Route,
    request::State,
    response::NamedFile
};
use ::rocket_contrib::Json;
use ::serde_json::{self, Value};

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

#[post("/_inbox", data = "<data>")]
fn inbox(data: Data, config: State<Config>, database: Database, signature: Result<ValidSignature, Error>) -> Result<Json<Value>, Error> {
    let _ = signature?;

    let mut data_str = String::new();
    data.open().read_to_string(&mut data_str)?;
    
    //println!("data_str:\n\n{}\n", data_str);

    let activity_json: Value = serde_json::from_str(&data_str)
        .map_err(Error::bad_request)?;

    //println!("activity_json: {:?}", activity_json);
    
    let new_activity = models::NewActivity::from_json(activity_json)
        .map_err(Error::bad_request)?;

    //println!("new_activity: {:?}", new_activity);

    ::diesel::insert_into(schema::inbox::table)
        .values(&new_activity)
        .execute(&database.conn)?;

    // TODO: this goes in a worker thread

    let activities = schema::inbox::table
        .order(schema::inbox::rowid.asc())
        .load::<models::Activity>(&database.conn)?;

    for activity in activities {
        match handle_activity(&config, &activity) {
            Ok(()) => {
                println!("handle_activity() succeeded");
            },
            Err(e) => {
                println!("handle_activity() failed: {:?}", e);
            }
        };

        let delete_q = schema::inbox::table.filter(
            schema::inbox::rowid.eq(activity.rowid)
        );

        ::diesel::delete(delete_q).execute(&database.conn)
            .map(|_| {
                println!("delete(rowid = {}) succeeded", activity.rowid);
            })
            .map_err(|e| {
                println!("delete(rowid = {}) failed: {:?}", activity.rowid, e);
                e
            })?;        
    }
    
    Ok(Json(Value::Null))
}

fn handle_activity(config: &Config, activity: &models::Activity) -> Result<(), ::failure::Error> {
    let json: Value = serde_json::from_str(&activity.json)?;
    
    let typ: String = json.get("type")
        .ok_or(format_err!("No 'type' field found"))?
        .as_str()
        .ok_or(format_err!("Invalid non-string 'type' field"))?
        .to_lowercase();

    let actor: &Value = json.get("actor")
        .ok_or(format_err!("No 'actor' field found"))?;

    let object = json.get("object")
        .ok_or(format_err!("No 'object' field found"))?;

    match &typ as &str {
        "follow" => {
            println!("follow!");

            let object_str = object.as_str()
                .ok_or(format_err!("Invalid non-string 'object' field"))?;
            
            if object_str != config.actor_url() {
                return Err(format_err!("Object is a different actor"));
            }

            // TODO: Store in followers collection
        },
        _ => {
            return Err(format_err!("Unsupported activity type: {}", typ));
        }
    };

    Ok(())
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

#[get("/_status")]
fn status(database: Database) -> Result<Json<Value>, Error> {
    let activities: Vec<Value> = schema::inbox::table
        .order(schema::inbox::rowid.desc())
        .load::<models::Activity>(&database.conn)?
        .iter()
        .map(|a| Ok(json!({
            "rowid": a.rowid,
            "id": a.id,
            "json": serde_json::from_str::<Value>(&a.json)?
        })))
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Json(json!({
        "inbox": activities
    })))
}

pub fn routes() -> Vec<Route> {
    routes![actor, inbox, outbox, media, status]
}
