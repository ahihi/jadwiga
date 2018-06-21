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

pub mod database;
pub mod models;
pub mod schema;

#[get("/<id>")]
fn index(id: Option<usize>) -> String {
    match id {
        Some(the_id) =>
            format!("Hello, world #{}!", the_id),
        None =>
            "hec".to_owned()
    }
}

pub fn run() {
    /*rocket::ignite()
        .mount("/", routes![index])
    .launch();*/

    /*
    let elem = post::Element::Html { content: "<h1>Wow</h1>".to_owned() };
    let bytes = bincode::serialize(&elem).unwrap();
    println!("{:?}", bytes);
    let e2: post::Element = bincode::deserialize(&bytes).unwrap();
    println!("{:?}", e2);
     */

    use schema::posts;
    
    let conn = database::connect().unwrap();

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
    
    let results = posts::table
        .load::<models::Post>(&conn)
        .unwrap();

    println!("{:?}", results);
}
