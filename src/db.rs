use ::diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use ::diesel::sqlite::SqliteConnection;
use ::failure::Error;
use ::rocket::http::Status;
use ::rocket::request::{self, FromRequest};
use ::rocket::{Request, State, Outcome};

use config::Config;

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

pub fn init_pool(config: &Config) -> Result<SqlitePool, Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(config.db_url.clone());
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
