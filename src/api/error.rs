use std::{error, fmt};

/// Possible errors that can arise during parsing and creating a cursor.
#[allow(dead_code)]
#[derive(Debug)]
pub enum ServiceError {
    ParseError(String),
    DatabaseError(mongodb::error::Error),
    ConnectionError(String),
    InvalidCursor(String),
    NotFound(String),
    Unknown(String),
}

impl From<ServiceError> for tonic::Status {
    fn from(err: ServiceError) -> tonic::Status {
        match err {
            ServiceError::ParseError(s) => tonic::Status::invalid_argument(s),
            ServiceError::NotFound(s) => tonic::Status::not_found(s),
            ServiceError::Unknown(s) => tonic::Status::unknown(s),
            ServiceError::InvalidCursor(s) => tonic::Status::out_of_range(s),
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}

impl From<mongodb::error::Error> for ServiceError {
    fn from(err: mongodb::error::Error) -> ServiceError {
        ServiceError::DatabaseError(err)
    }
}

impl From<&str> for ServiceError {
    fn from(message: &str) -> ServiceError {
        ServiceError::Unknown(message.to_owned())
    }
}

impl From<String> for ServiceError {
    fn from(message: String) -> ServiceError {
        ServiceError::Unknown(message)
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServiceError::DatabaseError(ref inner) => inner.fmt(fmt),
            ServiceError::InvalidCursor(ref cursor) => {
                write!(fmt, "Invalid cursor - unable to parse: {:?}", cursor)
            }
            ServiceError::ConnectionError(ref inner)
            | ServiceError::ParseError(ref inner)
            | ServiceError::NotFound(ref inner)
            | ServiceError::Unknown(ref inner) => inner.fmt(fmt),
        }
    }
}

#[allow(deprecated)]
impl error::Error for ServiceError {
    fn description(&self) -> &str {
        match *self {
            ServiceError::DatabaseError(ref inner) => inner.description(),
            ServiceError::InvalidCursor(_) => "Invalid cursor value",
            ServiceError::Unknown(ref inner)
            | ServiceError::ParseError(ref inner)
            | ServiceError::ConnectionError(ref inner)
            | ServiceError::NotFound(ref inner) => inner,
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            ServiceError::DatabaseError(ref inner) => Some(inner),
            _ => None,
        }
    }
}
