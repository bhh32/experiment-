//! Error types for oxy-sf.

use thiserror::Error;

/// The unified error type used throughout oxy-sf.
#[derive(Debug, Error)]
pub enum SfError {
    /// An authentication / OAuth failure.
    #[error("authentication error: {0}")]
    Auth(String),

    /// A transport / HTTP-level failure (connection, TLS, etc.).
    #[error("http error: {0}")]
    Http(String),

    /// A structured Salesforce REST API error.
    #[error("API error [{error_code}]: {message}")]
    Api {
        /// Human-readable message returned by Salesforce.
        message: String,
        /// Salesforce error code (e.g. `INVALID_FIELD`).
        error_code: String,
    },

    /// A configuration problem (bad project file, missing setting, ...).
    #[error("config error: {0}")]
    Config(String),

    /// An underlying I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON (de)serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// No target org could be resolved.
    #[error("no target org specified; pass --target-org or set a default org")]
    NoTargetOrg,

    /// The requested org/username is not authenticated locally.
    #[error("no authenticated org found for '{0}'")]
    NotAuthenticated(String),
}

impl SfError {
    /// A short, stable name for the error (used in JSON error envelopes).
    pub fn name(&self) -> &'static str {
        match self {
            SfError::Auth(_) => "AuthError",
            SfError::Http(_) => "HttpError",
            SfError::Api { .. } => "ApiError",
            SfError::Config(_) => "ConfigError",
            SfError::Io(_) => "IoError",
            SfError::Json(_) => "JsonError",
            SfError::NoTargetOrg => "NoTargetOrgError",
            SfError::NotAuthenticated(_) => "NotAuthenticatedError",
        }
    }

    /// A meaningful, non-zero process exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            SfError::Auth(_) => 2,
            SfError::Http(_) => 3,
            SfError::Api { .. } => 4,
            SfError::Config(_) => 5,
            SfError::Io(_) => 6,
            SfError::Json(_) => 7,
            SfError::NoTargetOrg => 8,
            SfError::NotAuthenticated(_) => 9,
        }
    }

    /// Parse a Salesforce REST error response body into an [`SfError::Api`].
    ///
    /// Salesforce returns errors as a JSON array of objects shaped like
    /// `[{"message": "...", "errorCode": "..."}]`. When the body cannot be
    /// parsed that way, the raw body is preserved in an [`SfError::Http`].
    pub fn from_rest_body(status: u16, body: &str) -> SfError {
        #[derive(serde::Deserialize)]
        struct RestError {
            #[serde(default)]
            message: String,
            #[serde(default, rename = "errorCode")]
            error_code: String,
        }

        if let Ok(errors) = serde_json::from_str::<Vec<RestError>>(body) {
            if let Some(first) = errors.into_iter().next() {
                return SfError::Api {
                    message: first.message,
                    error_code: if first.error_code.is_empty() {
                        "UNKNOWN_ERROR".to_string()
                    } else {
                        first.error_code
                    },
                };
            }
        }

        // Some endpoints (e.g. OAuth) return a single object instead.
        if let Ok(single) = serde_json::from_str::<RestError>(body) {
            if !single.message.is_empty() || !single.error_code.is_empty() {
                return SfError::Api {
                    message: single.message,
                    error_code: if single.error_code.is_empty() {
                        "UNKNOWN_ERROR".to_string()
                    } else {
                        single.error_code
                    },
                };
            }
        }

        SfError::Http(format!("HTTP {status}: {body}"))
    }
}

/// Convenience alias used across the crate.
pub type Result<T> = std::result::Result<T, SfError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_array_rest_error() {
        let body = r#"[{"message":"No such column 'Foo'","errorCode":"INVALID_FIELD"}]"#;
        let err = SfError::from_rest_body(400, body);
        match err {
            SfError::Api { message, error_code } => {
                assert_eq!(message, "No such column 'Foo'");
                assert_eq!(error_code, "INVALID_FIELD");
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn falls_back_to_http_for_unparseable() {
        let err = SfError::from_rest_body(500, "internal server error");
        assert!(matches!(err, SfError::Http(_)));
    }

    #[test]
    fn exit_codes_are_nonzero() {
        assert_ne!(SfError::NoTargetOrg.exit_code(), 0);
        assert_ne!(SfError::Auth("x".into()).exit_code(), 0);
    }
}
