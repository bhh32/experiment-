//! REST API helpers (SOQL query with automatic pagination).

use serde::Serialize;

use crate::connection::Connection;
use crate::error::{Result, SfError};

/// The aggregated result of a SOQL query (all pages combined).
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    /// Total number of records reported by Salesforce.
    pub total_size: u64,
    /// All records aggregated across `queryMore` pages.
    pub records: Vec<serde_json::Value>,
}

/// Run a SOQL query, following `nextRecordsUrl` until `done` is true.
pub fn query(conn: &mut Connection, soql: &str) -> Result<QueryResult> {
    let first_path = format!("{}/query?q={}", conn.data_path(), urlencode(soql));
    let mut value = conn.get_json(&first_path)?;

    let total_size = value
        .get("totalSize")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut records = take_records(&mut value)?;

    loop {
        let done = value.get("done").and_then(|v| v.as_bool()).unwrap_or(true);
        if done {
            break;
        }
        let Some(next) = value.get("nextRecordsUrl").and_then(|v| v.as_str()).map(String::from) else {
            break;
        };
        value = conn.get_json(&next)?;
        records.append(&mut take_records(&mut value)?);
    }

    Ok(QueryResult { total_size, records })
}

/// Remove and return the `records` array from a query response page.
fn take_records(value: &mut serde_json::Value) -> Result<Vec<serde_json::Value>> {
    match value.get_mut("records") {
        Some(serde_json::Value::Array(arr)) => Ok(std::mem::take(arr)),
        Some(_) => Err(SfError::Api {
            message: "query response 'records' field was not an array".into(),
            error_code: "MALFORMED_RESPONSE".into(),
        }),
        None => Ok(Vec::new()),
    }
}

/// Percent-encode a SOQL string for use in a query parameter.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::store::AuthInfo;
    use httpmock::prelude::*;

    fn conn_for(base: &str) -> Connection {
        let info = AuthInfo {
            username: "u@example.com".into(),
            instance_url: base.to_string(),
            access_token: "tok".into(),
            refresh_token: None,
            client_id: "PlatformCLI".into(),
            login_url: base.to_string(),
            alias: None,
            org_id: None,
        };
        Connection::from_auth(&info, Some("62.0")).unwrap()
    }

    #[test]
    fn paginates_query_more() {
        let server = MockServer::start();

        let _first = server.mock(|when, then| {
            when.method(GET).path("/services/data/v62.0/query");
            then.status(200).json_body(serde_json::json!({
                "totalSize": 3,
                "done": false,
                "nextRecordsUrl": "/services/data/v62.0/query/01g000-2000",
                "records": [{"Id": "1"}, {"Id": "2"}]
            }));
        });

        let _second = server.mock(|when, then| {
            when.method(GET).path("/services/data/v62.0/query/01g000-2000");
            then.status(200).json_body(serde_json::json!({
                "totalSize": 3,
                "done": true,
                "records": [{"Id": "3"}]
            }));
        });

        let mut conn = conn_for(&server.base_url());
        let result = query(&mut conn, "SELECT Id FROM Account").unwrap();
        assert_eq!(result.total_size, 3);
        assert_eq!(result.records.len(), 3);
        assert_eq!(result.records[2]["Id"], "3");
    }

    #[test]
    fn surfaces_api_error() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/services/data/v62.0/query");
            then.status(400).json_body(serde_json::json!([
                {"message": "bad field", "errorCode": "INVALID_FIELD"}
            ]));
        });
        let mut conn = conn_for(&server.base_url());
        let err = query(&mut conn, "SELECT Bogus FROM Account").unwrap_err();
        match err {
            SfError::Api { error_code, .. } => assert_eq!(error_code, "INVALID_FIELD"),
            other => panic!("expected Api error, got {other:?}"),
        }
    }
}
