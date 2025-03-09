pub enum Error {
    FailedHeaderConversion,
    InvalidAuthentificationFormat,
    DecodeError,
    ConversionError,
    TransmissionError,
    InvalidDataFormat,
    QueryParsingError,
    QueryNotProvided,
    QueryInvalidAPI,
    DatabaseQueryNotSupported,
    DatabaseQueryInvalidFormat,
    DatabaseConfigError(String),
    DatabaseQueryFailed(String),
    DatabaseDeletionError(String),
    DatabaseUpdateError(String)
}

pub trait Logging {
    type Path: ?Sized;
    type Output;

    fn log_to_cli(self) -> Self::Output;

    fn log_to_file(self, path: &Self::Path) -> Self::Output;
}
