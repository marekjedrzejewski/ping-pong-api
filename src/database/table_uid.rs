use std::fmt;

use sqlx::Type;

// IMPORTANT: Remember to keep this in sync with the database!
// I wanted to avoid duplicating these rules, but it seems like that would get into
// overengineering pretty quickly.
const UID_MAX_LENGTH: usize = 6;
fn is_valid_uid(uid: &str) -> bool {
    uid.len() <= UID_MAX_LENGTH
        && !uid.is_empty()
        && uid
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

#[derive(Clone, Hash, PartialEq, Eq, Type, Debug)]
#[sqlx(transparent)]
pub struct TableUid(String);

impl TableUid {
    /// Parses a string into a TableUid, validating the format (lowercase + digits, max 6 chars)
    pub fn parse(uid: impl Into<String>) -> Result<Self, TableUidError> {
        let uid = uid.into();
        if !is_valid_uid(&uid) {
            return Err(TableUidError::InvalidFormat);
        }
        Ok(Self(uid))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TableUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_uids() {
        let valid_uids = ["abc123", "a1b2c3", "123456", "abcdef", "a", "1"];
        for &uid in &valid_uids {
            assert_eq!(TableUid::parse(uid).unwrap().as_str(), uid);
        }
    }

    #[test]
    fn invalid_uids() {
        let invalid_uids = ["toolong", "UPPER", "b-a-d!", "", "💥"];
        for uid in invalid_uids {
            assert!(matches!(
                TableUid::parse(uid),
                Err(TableUidError::InvalidFormat)
            ));
        }
    }

    #[test]
    fn display_implementation() {
        let uid_str = "abc123";
        let uid = TableUid::parse(uid_str).unwrap();
        assert_eq!(uid.to_string(), uid_str);

        let err = TableUidError::InvalidFormat;
        assert!(err.to_string().contains(&UID_MAX_LENGTH.to_string()));
    }
}
