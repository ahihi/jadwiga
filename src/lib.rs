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
        .mount("/", api::activitypub::routes())
        .mount("/", api::webfinger::routes())
        .launch();

    Ok(())
}
