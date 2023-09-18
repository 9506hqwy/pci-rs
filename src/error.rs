use nom;

#[derive(Debug)]
pub enum Error {
    Parse(String),
    TrailingData,
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error {
    fn from(error: nom::Err<nom::error::Error<&'a str>>) -> Self {
        Error::Parse(format!("{}", error))
    }
}
