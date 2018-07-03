use std::io::Write;

use ::bincode;
use ::diesel::backend::Backend;
use ::diesel::deserialize::{self, FromSql};
use ::diesel::serialize::{self, Output, ToSql};
use ::diesel::sql_types::Binary;

use ::schema::posts;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Piece {
    Html(String),
    MoreVariantsToCome
}

#[derive(Debug, PartialEq, Serialize, Deserialize, FromSqlRow, AsExpression)]
#[sql_type = "Binary"]
pub struct Body {
    pub pieces: Vec<Piece>
}

impl<DB: Backend> ToSql<Binary, DB> for Body {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        let bytes = bincode::serialize(&self)?;
        <_ as ToSql<Binary, DB>>::to_sql(&bytes, out)
    }
}

impl<DB: Backend> FromSql<Binary, DB> for Body
    where *const [u8]: FromSql<Binary, DB>
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let bytes_ptr = <*const [u8] as FromSql<Binary, DB>>::from_sql(bytes)?;
        let bytes_ref = unsafe { &*bytes_ptr };
        let body = bincode::deserialize(bytes_ref)?;
        Ok(body)
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Post {
    pub id: i32,
    pub uri_name: String,
    pub datetime: i32,
    pub title: String,
    pub body: Body
}

#[derive(Debug, Insertable)]
#[table_name="posts"]
pub struct NewPost {
    pub uri_name: String,
    pub title: String,
    pub body: Body
}
