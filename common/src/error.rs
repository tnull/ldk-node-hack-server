use prost::DecodeError;

#[derive(Debug)]
pub enum ServerHackError {
	FailedRequest(reqwest::StatusCode, Vec<u8>),
	InternalError(String),
}

impl From<DecodeError> for ServerHackError {
	fn from(err: DecodeError) -> Self {
		ServerHackError::InternalError(err.to_string())
	}
}

impl From<reqwest::Error> for ServerHackError {
	fn from(err: reqwest::Error) -> Self {
		ServerHackError::InternalError(err.to_string())
	}
}
