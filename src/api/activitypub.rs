use ::activitypub::{
    actor::Person,
    collection::OrderedCollection,
    context,
    object::Note
};
use ::diesel::prelude::*;
use ::failure::Error;
use ::rocket::Route;
use ::rocket::request::State;
use ::rocket::response::status::NotFound;
use ::rocket_contrib::Json;

use config::Config;
use db::Database;
use models;
use schema;

fn ap_run<T, F>(act: F) -> Result<Json<T>, NotFound<String>>
    where F: Fn() -> Result<T, Error>
{
    act()
        .map(|data| Json(data))
        .map_err(|e| NotFound(format!("{}", e)))
}

fn get_actor(config: &Config, _database: &Database) -> Result<Person, Error> {
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

    person.ap_actor_props.set_inbox_string(
        config.inbox_url().to_owned()
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

fn get_outbox(database: &Database) -> Result<OrderedCollection, Error> {
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
fn actor(config: State<Config>, database: Database) -> Result<Json<Person>, NotFound<String>> {
    ap_run(|| get_actor(&config, &database))
}

#[get("/_outbox")]
fn outbox(database: Database) -> Result<Json<OrderedCollection>, NotFound<String>> {
    ap_run(|| get_outbox(&database))
}

pub fn routes() -> Vec<Route> {
    routes![actor, outbox]
}
