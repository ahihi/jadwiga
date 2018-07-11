use ::std::io;
use ::std::path::{Path, PathBuf};
use ::std::str::FromStr;

use ::chrono::offset::{TimeZone, Utc};
use ::diesel::prelude::*;
use ::rocket::{
    Route,
    http::Status,
    outcome::Outcome,
    request::{
        self,
        FromRequest,
        Request,
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
use parser;
use schema;

mod ns {
    pub const ACTIVITYSTREAMS: &str = "https://www.w3.org/ns/activitystreams";
    pub const PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

    pub const SECURITY: &str = "https://w3id.org/security/v1";
}

#[derive(Debug)]
pub struct Signature {
    key_id: String,
    headers: Vec<String>,
    signature: String
}

impl FromStr for Signature {
    type Err = ::failure::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields = parser::sig(s)?;

        let key_id = match fields.get("keyId") {
            Some(text) => text,
            None => return Err(format_err!("No 'keyId' field found"))
        };

        let headers = match fields.get("headers") {
            Some(text) => parser::sig_headers(text)?,
            None => return Err(format_err!("No 'headers' field found"))
        };

        let signature = match fields.get("signature") {
            Some(text) => text,
            None => return Err(format_err!("No 'signature' field found"))
        };

        Ok(Signature {
            key_id: key_id.to_owned(),
            headers: headers,
            signature: signature.to_owned()
        })
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Signature {
    type Error = ::failure::Error;
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let header = match request.headers().get_one("Signature") {
            Some(h) => h,
            None => {
                let e = format_err!("No 'Signature' header found");
                println!("e: {}", e);
                return Outcome::Failure((Status::BadRequest, e))
            }
        };

        let signature = match header.parse::<Signature>() {
            Ok(s) => s,
            Err(e) => {
                println!("e: {}", e);
                return Outcome::Failure((Status::BadRequest, e))
            }
        };

        println!("map: {:?}", signature);

        Outcome::Success(signature)
    }
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
fn inbox(config: State<Config>, database: Database, signature: Signature) -> Result<Json<Value>, Error> {

    println!("{:?}", signature);

    Ok(Json(Value::Null))
    /*
signature_header = request.headers['Signature'].split(',').map do |pair|
    pair.split('=').map do |value|
      value.gsub(/\A"/, '').gsub(/"\z/, '') # "foo" -> foo
    end
  end.to_h

  key_id    = signature_header['keyId']
  headers   = signature_header['headers']
  signature = Base64.decode64(signature_header['signature'])

  actor = JSON.parse(HTTP.get(key_id).to_s)
  key   = OpenSSL::PKey::RSA.new(actor['publicKey']['publicKeyPem'])

  comparison_string = headers.split(' ').map do |signed_header_name|
    if signed_header_name == '(request-target)'
      '(request-target): post /inbox'
    else
      "#{signed_header_name}: #{request.headers[signed_header_name.capitalize]}"
    end
  end

  if key.verify(OpenSSL::Digest::SHA256.new, signature, comparison_string)
    request.body.rewind
    INBOX << request.body.read
    [200, 'OK']
  else
    [401, 'Request signature could not be verified']
  end
     */
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
