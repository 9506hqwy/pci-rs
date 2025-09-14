use nom;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    NotFoundAcpiMcfg,
    Parse(String),
    TrailingData,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error {
    fn from(error: nom::Err<nom::error::Error<&'a str>>) -> Self {
        Error::Parse(format!("{error}"))
    }
}
