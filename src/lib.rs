#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate activitypub;
extern crate bincode;
#[macro_use] extern crate diesel;
extern crate failure;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate url;
extern crate webfinger;

use diesel::prelude::*;
use failure::Error;
use rocket_contrib::Json;

pub mod api;
pub mod config;
pub mod db;
pub mod models;
pub mod schema;

use config::Config;

#[get("/<id>")]
fn index(id: Option<i32>, database: db::Database) -> String {
    match id {
        Some(the_id) => {
            let results = schema::posts::table
                .filter(schema::posts::id.eq(the_id))
                .load::<models::Post>(&database.conn)
                .unwrap();

            format!("{:?}", results).to_owned()
        },
        None =>
            "hec".to_owned()
    }
}

use activitypub::{
    actor::Person,
    collection::OrderedCollection,
    context,
    object::{Note}
};
use rocket::Route;
use rocket::request::{State};
use rocket::response::status::NotFound;

fn ap_run<T, F>(act: F) -> Result<Json<T>, NotFound<String>>
    where F: Fn() -> Result<T, Error>
{
    act()
        .map(|data| Json(data))
        .map_err(|e| NotFound(format!("{}", e)))
}

fn get_actor(config: &Config, database: &db::Database) -> Result<Person, Error> {
    let mut person = Person::default();
    
    person.object_props.set_context_object(
        context()
    )?;

    let id = config.actor_url();
    person.object_props.set_id_string(
        id.to_owned()
    )?;
    
    person.object_props.set_name_string(
        config.actor_name.clone()
    )?;

    person.ap_actor_props.set_preferred_username_string(
        config.actor_username.clone()
    )?;
    
    person.ap_actor_props.set_outbox_string(
        config.outbox_url().to_owned()
    )?;
    
    Ok(person)
}

fn get_note(post: &models::Post) -> Result<Note, Error> {
    let mut note = Note::default();

    note.object_props.set_id_string(
        post.id.to_string()
    )?;

    Ok(note)
}

fn get_outbox(database: &db::Database) -> Result<OrderedCollection, Error> {
    let posts = schema::posts::table
        .load::<models::Post>(&database.conn)?;
    
    let mut outbox = OrderedCollection::default();

    outbox.object_props.set_context_object(
        context()
    )?;

    let items = posts.iter()
        .map(get_note)
        .collect::<Result<Vec<_>, _>>()?;
    outbox.collection_props.set_items_object_vec(
        items
    )?;

    Ok(outbox)
}

#[get("/")]
fn actor(config: State<Config>, database: db::Database) -> Result<Json<Person>, NotFound<String>> {
    ap_run(|| get_actor(&config, &database))
}

#[get("/_outbox")]
fn outbox(database: db::Database) -> Result<Json<OrderedCollection>, NotFound<String>> {
    ap_run(|| get_outbox(&database))
}

// Mastodon

#[get("/accounts/<id>")]
fn account(id: Option<String>, config: State<Config>, database: db::Database) -> Result<Json<Person>, NotFound<String>> {
    match id {
        Some(id) =>
            if id == config.actor_username {
                ap_run(|| get_actor(&config, &database))
            } else {
                Err(NotFound("nyoro~n".to_owned()))
            },
        None => unimplemented!()
    }
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let pool = db::init_pool(&config)?;
    
    /*
    let new_post = models::NewPost {
        uri_name: "test".to_owned(),
        title: "Test Post".to_owned(),
        body: models::Body {
            pieces: vec![]
        }
    };
    diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(&conn)
        .unwrap();
     */

    let ap_routes = routes![actor, outbox];
    let webfinger_routes = api::webfinger::routes();
    let mastodon_routes = routes![account];

    rocket::ignite()
        .manage(config)
        .manage(pool)
        .mount("/", ap_routes)
        .mount("/", webfinger_routes)
        .mount("/api/v1/", mastodon_routes)
        .launch();

    Ok(())
}
