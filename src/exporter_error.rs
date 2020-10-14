use failure::Fail;

#[derive(Debug, Fail)]
pub enum ExporterError {
    #[fail(display = "IO Error: {}", e)]
    IO { e: std::io::Error },

    #[fail(display = "UTF-8 Error: {}", e)]
    UTF8 { e: std::str::Utf8Error },

    #[fail(display = "Parse Int Error: {}", e)]
    ParseInt { e: std::num::ParseIntError },
}

impl From<std::io::Error> for ExporterError {
    fn from(e: std::io::Error) -> Self {
        ExporterError::IO { e }
    }
}

impl From<std::str::Utf8Error> for ExporterError {
    fn from(e: std::str::Utf8Error) -> Self {
        ExporterError::UTF8 { e }
    }
}

impl From<std::num::ParseIntError> for ExporterError {
    fn from(e: std::num::ParseIntError) -> Self {
        ExporterError::ParseInt { e }
    }
}
