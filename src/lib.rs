#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate bincode;
extern crate chrono;
#[macro_use] extern crate diesel;
extern crate failure;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use ]extern crate serde_json;
extern crate url;
extern crate webfinger;

use failure::Error;

pub mod api;
pub mod config;
pub mod db;
pub mod models;
pub mod schema;

use config::Config;

pub fn run(config: Config) -> Result<(), Error> {
    let pool = db::init_pool(&config)?;

    /*
    let conn = pool.get()?;
    
    use diesel::prelude::*;

    let new_post = models::NewPost {
        uri_name: "test_post".to_owned(),
        title: "Test Post".to_owned(),
        body: models::Body {
            pieces: vec![
                models::Piece::Html("<strong>hewwo!!!</strong>".to_owned())
            ]
        }
    };
    diesel::insert_into(schema::posts::table)
        .values(&new_post)
        .execute(&conn)
        .unwrap()
        ;

    return Ok(());
     */

    rocket::ignite()
        .manage(config)
        .manage(pool)
        .mount("/", api::activitypub::routes())
        .mount("/", api::webfinger::routes())
        .launch();

    Ok(())
}
