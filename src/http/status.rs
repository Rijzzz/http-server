#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    SwitchingProtocols,
    Ok,
    Created,
    NoContent,
    PartialContent,
    MovedPermanently,
    Found,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    PayloadTooLarge,
    RangeNotSatisfiable,
    InternalServerError,
    NotImplemented,
}

impl StatusCode {
    #[must_use]
    pub fn as_u16(self) -> u16 {
        match self {
            Self::SwitchingProtocols => 101,
            Self::Ok => 200,
            Self::Created => 201,
            Self::NoContent => 204,
            Self::PartialContent => 206,
            Self::MovedPermanently => 301,
            Self::Found => 302,
            Self::NotModified => 304,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::PayloadTooLarge => 413,
            Self::RangeNotSatisfiable => 416,
            Self::InternalServerError => 500,
            Self::NotImplemented => 501,
        }
    }

    #[must_use]
    pub fn reason(self) -> &'static str {
        match self {
            Self::SwitchingProtocols => "Switching Protocols",
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::NoContent => "No Content",
            Self::PartialContent => "Partial Content",
            Self::MovedPermanently => "Moved Permanently",
            Self::Found => "Found",
            Self::NotModified => "Not Modified",
            Self::BadRequest => "Bad Request",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::PayloadTooLarge => "Payload Too Large",
            Self::RangeNotSatisfiable => "Range Not Satisfiable",
            Self::InternalServerError => "Internal Server Error",
            Self::NotImplemented => "Not Implemented",
        }
    }
}
