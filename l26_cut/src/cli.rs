pub use clap::Parser;

use std::{
    error::Error, 
    fmt::{self, Display, Formatter}, 
    num::ParseIntError
};

#[derive(Parser)]
pub struct Cli {
    /// выбрать поля (колонки)
    #[clap(
        short, 
        num_args = 1..,
        required = true, 
        value_delimiter = ',',
        value_parser = field_parser,
    )]
    pub fields: Vec<u32>,

    /// использовать другой разделитель
    #[clap(short, default_value_t = '\t')]
    pub delimiter: char,

    /// только строки с разделителем
    #[clap(short)]
    pub separated: bool,
}


fn field_parser(str: &str) -> Result<u32, ParseFieldError> {
    match str.parse().map_err(ParseFieldError::ParseIntError)? {
        0 => Err(ParseFieldError::LessThanOne),
        value => Ok(value - 1)
    }
}

#[derive(Debug)]
enum ParseFieldError {
    ParseIntError(ParseIntError),
    LessThanOne,
}

impl Display for ParseFieldError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ParseFieldError::*;

        match self {
            ParseIntError(err) => write!(f, "{}", err),
            LessThanOne => write!(f, "value must be greater or equal to 1"),
        }
    }
}

impl Error for ParseFieldError {}
