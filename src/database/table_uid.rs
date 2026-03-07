use std::{fmt, ops::Deref};

use sqlx::Type;

// IMPORTANT: Remember to keep this in sync with the database!
// I wanted to avoid duplicating these rules, but it seems like that would get into
// overengineering pretty quickly.
const UID_MAX_LENGTH: usize = 6;
fn is_valid_uid(uid: &str) -> bool {
    uid.len() <= UID_MAX_LENGTH
        && uid
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

#[derive(Clone, Hash, PartialEq, Eq, Type, Debug)]
#[sqlx(transparent)]
pub struct TableUid(String);

impl TableUid {
    pub fn parse(uid: impl Into<String>) -> Result<Self, TableUidError> {
        let uid = uid.into();
        if !is_valid_uid(&uid) {
            return Err(TableUidError::InvalidFormat);
        }
        Ok(Self(uid))
    }
}

impl Deref for TableUid {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for TableUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug)]
pub enum TableUidError {
    InvalidFormat,
}

impl fmt::Display for TableUidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableUidError::InvalidFormat => write!(
                f,
                "Invalid Table UID format. Must be at most {} characters long and contain only lowercase letters and digits.",
                UID_MAX_LENGTH
            ),
        }
    }
}
