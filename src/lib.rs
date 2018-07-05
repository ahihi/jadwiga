#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate bincode;
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
        .mount("/", api::activitypub::routes())
        .mount("/", api::webfinger::routes())
        .launch();

    Ok(())
}
