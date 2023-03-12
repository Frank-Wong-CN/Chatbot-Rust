use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ArgumentError {
	pub argument: String,
	message: String
}

impl ArgumentError {
	pub fn new(arg: &str, msg: &str) -> Self {
		Self { argument: arg.into(), message: msg.into() }
	}
}

impl Display for ArgumentError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Error when reading argument {}: {}", self.argument, self.message)
	}
}

impl Error for ArgumentError {}

#[derive(Debug)]
pub enum MainError {
	ArgumentError(ArgumentError),

	IOError(std::io::Error),
	SQLiteError(rusqlite::Error),
}

impl Display for MainError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ArgumentError(err) => write!(f, "{}", err),
			Self::IOError(err) => write!(f, "{}", err),
			Self::SQLiteError(err) => write!(f, "{}", err),
		}
	}
}

impl From::<ArgumentError> for MainError {
    fn from(value: ArgumentError) -> Self {
        return Self::ArgumentError(value)
    }
}

impl From::<std::io::Error> for MainError {
    fn from(value: std::io::Error) -> Self {
        return Self::IOError(value)
    }
}

impl From::<rusqlite::Error> for MainError {
	fn from(value: rusqlite::Error) -> Self {
		return Self::SQLiteError(value)
	}
}
