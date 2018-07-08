use ::std::fmt::{self, Debug, Display, Formatter};
use ::std::io::{self, Cursor};

use ::rocket::{
    Request, Response,
    http::{ContentType, Status},
    response::Responder
};

#[derive(Debug)]
pub enum Error {
    NotFound,
    Internal(String)
}

impl Error {
    pub fn from_io(e: io::Error) -> Error {
        match e.kind() {
            io::ErrorKind::NotFound =>
                Error::NotFound,
            other =>
                Error::Internal(format!("{:?}", e))
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self, f)
    }
}

impl<E: ::std::error::Error> From<E> for Error {
    fn from(e: E) -> Self {
        Error::Internal(format!("{:?}", e))
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _request: &Request) -> Result<Response<'r>, Status> {
        let (status, message): (Status, String) = match self {
            Error::NotFound =>
                (Status::NotFound, "4w4 what's this?".to_owned()),
            Error::Internal(why) => (
                Status::InternalServerError,
                format!("{}\n\n{}", "OOPSIE WOOPSIE!! Uwu We made a fucky wucky!! A wittle fucko boingo! The code monkeys at our headquarters are working VEWY HAWD to fix this!", why)
            )
        };
        
        Response::build()
            .status(status)
            .header(ContentType::Plain)
            .sized_body(Cursor::new(message.into_bytes()))
            .ok()
    }
}
