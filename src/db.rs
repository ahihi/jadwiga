use std::env;

use ::diesel::prelude::*;
use ::diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use ::diesel::sqlite::SqliteConnection;
use ::dotenv::dotenv;
use ::failure::Error;
use ::rocket::http::Status;
use ::rocket::request::{self, FromRequest};
use ::rocket::{Request, State, Outcome};

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

pub fn init_pool() -> Result<SqlitePool, Error> {
    dotenv()?;

    let db_url = env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::new(manager)?;

    Ok(pool)
}

pub struct Database {
    pub conn: PooledConnection<ConnectionManager<SqliteConnection>>
}

impl<'a, 'r> FromRequest<'a, 'r> for Database {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Database, ()> {
        let pool = request.guard::<State<SqlitePool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Database { conn: conn }),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}
