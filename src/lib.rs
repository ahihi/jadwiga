#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate activitypub;
extern crate bincode;
#[macro_use] extern crate diesel;
extern crate failure;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate url;

use std::error::Error;

use diesel::prelude::*;
use rocket_contrib::Json;

pub mod config;
pub mod db;
pub mod models;
pub mod schema;

use config::Config;
use schema::posts;

#[get("/<id>")]
fn index(id: Option<i32>, database: db::Database) -> String {
    match id {
        Some(the_id) => {
            let results = posts::table
                .filter(posts::id.eq(the_id))
                .load::<models::Post>(&database.conn)
                .unwrap();

            format!("{:?}", results).to_owned()
        },
        None =>
            "hec".to_owned()
    }
}

use activitypub::{context, object::{Note, Profile}};
use rocket::request::{State};
use rocket::response::status::NotFound;

fn get_profile(config: State<Config>, database: db::Database) -> Result<Profile, activitypub::Error> {
    let mut profile = Profile::default();
    
    profile.object_props.set_context_object(context())?;

    let id = config.profile_url();
    profile.object_props.set_id_string(id.to_owned())?;
    
    profile.object_props.set_name_string(config.ap_username.clone())?;

    Ok(profile)
}

#[get("/")]
fn profile(config: State<Config>, database: db::Database) -> Result<Json<Profile>, NotFound<String>> {
    let profile = get_profile(config, database)
        .map_err(|e| NotFound(format!("{}", e)))?;

    Ok(Json(profile))
}

#[get("/outbox")]
fn outbox(database: db::Database) -> Result<Json<Note>, NotFound<String>> {
    let results = posts::table
        .load::<models::Post>(&database.conn)
        .map_err(|e| NotFound(format!("{}", e)))?;

    let mut note = Note::default();
    note.object_props.set_context_object(context())
        .map_err(|e| NotFound(format!("{}", e)))?;
    
    Ok(Json(note))
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
    
    rocket::ignite()
        .manage(config)
        .manage(pool)
        .mount("/", routes![profile, outbox])
        .launch();

    Ok(())
}
