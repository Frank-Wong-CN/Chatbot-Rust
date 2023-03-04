#[derive(Debug)]
pub enum MainError {
	IOError(std::io::Error),
	SQLiteError(rusqlite::Error)
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
