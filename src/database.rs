use std::env;

use ::diesel::prelude::*;
use ::diesel::sqlite::SqliteConnection;
use ::dotenv::dotenv;
use ::failure::Error;

pub fn connect() -> Result<SqliteConnection, Error> {
    dotenv()?;

    let db_url = env::var("DATABASE_URL")?;
    let conn = SqliteConnection::establish(&db_url)?;

    Ok(conn)
}
