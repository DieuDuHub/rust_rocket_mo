use std::error::Error;
use std::fmt;

#[macro_export]
macro_rules! local_error {
    // match something(q,r,t,6,7,8) etc
    // compiler extracts function name and arguments. It injects the values in respective varibles.
    ($a:path,$b:expr) => {
        ApiError::new($a($b.to_string()))
    };
}

#[derive(Clone, Debug)]
pub enum LocalError {
    ParsingError(String),
    OidFormatError(String),
    DataNotFoundError(String),
    ConnectionError(String),
    ContextError(String),
    FilterDateParsing(String),
    FilterStringarsing(String),
}

impl fmt::Display for LocalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LocalError::ParsingError(desc) => write!(f, "Parsing exception : {}", desc),
            LocalError::OidFormatError(desc) => write!(f, "ObjectId exception : {}", desc),
            LocalError::ConnectionError(desc) => write!(f, "Connection exception : {}", desc),
            LocalError::DataNotFoundError(desc) => write!(f, "Data not found : {}", desc),
            LocalError::ContextError(desc) => write!(f, "Context exception : {}", desc),
            LocalError::FilterDateParsing(desc) => write!(f, "Filter exception : {}", desc),
            LocalError::FilterStringarsing(desc) => write!(f, "Filter exception : {}", desc),
            // _ => write!(f, "Global exception"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiError {
    error: LocalError,
    description: String, // NEW
}

// NEW
impl ApiError {
    pub fn new(error: LocalError) -> Self {
        ApiError {
            description: error.to_string(),
            error,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Error for ApiError {
    fn description(&self) -> &str {
        &self.description
    }
}
