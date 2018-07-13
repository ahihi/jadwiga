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
    BadRequest(String),
    Internal(String)
}

impl Error {
    pub fn bad_request<E: Debug>(e: E) -> Self {
        Error::BadRequest(format!("{:?}", e))
    }
    
    pub fn internal<E: Debug>(e: E) -> Self {
        Error::Internal(format!("{:?}", e))
    }

    pub fn from_io(e: io::Error) -> Error {
        match e.kind() {
            io::ErrorKind::NotFound =>
                Error::NotFound,
            _ =>
                Error::internal(e)
        }
    }

    pub fn status(&self) -> Status {
        match self {
            Error::NotFound => Status::NotFound,
            Error::BadRequest(_) => Status::BadRequest,
            Error::Internal(_) => Status::InternalServerError
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
        Error::internal(e)
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _request: &Request) -> Result<Response<'r>, Status> {
        let message = match self {
            Error::NotFound =>
                "4w4 what's this?".to_owned(),
            Error::BadRequest(ref why) =>
                format!("{}\n\n{}", "Bad request", why),
            Error::Internal(ref why) => 
                format!("{}\n\n{}", "OOPSIE WOOPSIE!! Uwu We made a fucky wucky!! A wittle fucko boingo! The code monkeys at our headquarters are working VEWY HAWD to fix this!", why)
        };
        
        Response::build()
            .status(self.status())
            .header(ContentType::Plain)
            .sized_body(Cursor::new(message.into_bytes()))
            .ok()
    }
}
