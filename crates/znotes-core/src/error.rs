use serde::ser::{Serialize, SerializeStruct, Serializer};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("wrong password")]
    WrongPassword,
    #[error("not a znotes vault: {0}")]
    NotAVault(String),
    #[error("a vault already exists at {0}")]
    AlreadyExists(String),
    #[error("vault format {found} is newer than the supported {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("vault is locked")]
    Locked,
    #[error("{entity} {id} not found")]
    NotFound { entity: &'static str, id: i64 },
    #[error("{0}")]
    InvalidInput(String),
    #[error("key derivation failed: {0}")]
    Kdf(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::WrongPassword => "wrong_password",
            Error::NotAVault(_) => "not_a_vault",
            Error::AlreadyExists(_) => "already_exists",
            Error::UnsupportedVersion { .. } => "unsupported_version",
            Error::Locked => "locked",
            Error::NotFound { .. } => "not_found",
            Error::InvalidInput(_) => "invalid_input",
            Error::Kdf(_) => "kdf",
            Error::Io(_) => "io",
            Error::Db(_) => "db",
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Error::InvalidInput(message.into())
    }
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Error", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_to_code_and_message() {
        let json = serde_json::to_value(Error::WrongPassword).unwrap();
        assert_eq!(json["code"], "wrong_password");
        assert_eq!(json["message"], "wrong password");
    }

    #[test]
    fn every_variant_has_a_snake_case_code() {
        let variants = [
            Error::WrongPassword,
            Error::NotAVault("x".into()),
            Error::AlreadyExists("x".into()),
            Error::UnsupportedVersion {
                found: 2,
                supported: 1,
            },
            Error::Locked,
            Error::NotFound {
                entity: "note",
                id: 1,
            },
            Error::invalid("x"),
            Error::Kdf("x".into()),
            Error::Io(std::io::Error::other("x")),
            Error::Db(rusqlite::Error::InvalidQuery),
        ];
        for variant in variants {
            let code = variant.code();
            assert!(
                code.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{code}"
            );
        }
    }
}
