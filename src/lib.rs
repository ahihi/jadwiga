#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate bincode;
#[macro_use] extern crate diesel;
extern crate dotenv;
extern crate failure;
extern crate rocket;
extern crate serde;
#[macro_use] extern crate serde_derive;

use diesel::prelude::*;

pub mod db;
pub mod models;
pub mod schema;

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

pub fn run() {
    /*
    let elem = post::Element::Html { content: "<h1>Wow</h1>".to_owned() };
    let bytes = bincode::serialize(&elem).unwrap();
    println!("{:?}", bytes);
    let e2: post::Element = bincode::deserialize(&bytes).unwrap();
    println!("{:?}", e2);
     */

    let pool = db::init_pool().unwrap();
    
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
    
    /*
    let results = posts::table
        .load::<models::Post>(&conn)
        .unwrap();

    println!("{:?}", results);
    */

    rocket::ignite()
        .manage(pool)
        .mount("/", routes![index])
        .launch();
}
