use axum::http::HeaderMap;
use std::collections::HashSet;
use subtle::ConstantTimeEq;

pub enum SecurityCheckResult {
    Allowed { token_in_sec_ws: bool },
    Denied(&'static str),
}

pub fn check_auth<S: std::hash::BuildHasher>(
    headers: &HeaderMap,
    configured_secret: &str,
    allowed_origins: &HashSet<String, S>,
) -> SecurityCheckResult {
    if let Some(auth_val) = headers.get(axum::http::header::AUTHORIZATION)
        && let Ok(auth_str) = auth_val.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        let token = token.trim();
        return if is_valid_secret(token, configured_secret) {
            SecurityCheckResult::Allowed {
                token_in_sec_ws: false,
            }
        } else {
            SecurityCheckResult::Denied("Invalid API key")
        };
    }

    if let Some(ws_proto_val) = headers.get("sec-websocket-protocol")
        && let Ok(ws_proto_str) = ws_proto_val.to_str()
        && let Some(token) = ws_proto_str.strip_prefix("minecraft-v1,")
    {
        let token = token.trim();
        let is_origin_allowed = headers
            .get(axum::http::header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|origin| {
                !origin.is_empty()
                    && (allowed_origins.contains(origin) || allowed_origins.contains("*"))
            });

        if !is_origin_allowed {
            return SecurityCheckResult::Denied("Origin Not Allowed");
        }

        return if is_valid_secret(token, configured_secret) {
            SecurityCheckResult::Allowed {
                token_in_sec_ws: true,
            }
        } else {
            SecurityCheckResult::Denied("Invalid API key")
        };
    }

    SecurityCheckResult::Denied("Missing API key")
}

#[must_use]
pub fn is_valid_secret(supplied: &str, configured: &str) -> bool {
    if supplied.is_empty() || supplied.len() != configured.len() {
        return false;
    }
    supplied.as_bytes().ct_eq(configured.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn validates_constant_time_secret() {
        assert!(is_valid_secret("secret123", "secret123"));
        assert!(!is_valid_secret("secret124", "secret123"));
        assert!(!is_valid_secret("short", "secret123"));
        assert!(!is_valid_secret("", "secret123"));
        assert!(!is_valid_secret("", ""));
    }

    #[test]
    fn allows_valid_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer test_secret_12345"),
        );
        let origins = HashSet::new();
        match check_auth(&headers, "test_secret_12345", &origins) {
            SecurityCheckResult::Allowed { token_in_sec_ws } => assert!(!token_in_sec_ws),
            SecurityCheckResult::Denied(_) => panic!("Expected allowed"),
        }
    }

    #[test]
    fn denies_invalid_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer wrong_secret"),
        );
        let origins = HashSet::new();
        match check_auth(&headers, "test_secret_12345", &origins) {
            SecurityCheckResult::Denied(reason) => assert_eq!(reason, "Invalid API key"),
            SecurityCheckResult::Allowed { .. } => panic!("Expected denied"),
        }
    }

    #[test]
    fn allows_websocket_protocol_with_matching_origin() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "sec-websocket-protocol",
            HeaderValue::from_static("minecraft-v1, test_secret_12345"),
        );
        headers.insert(
            axum::http::header::ORIGIN,
            HeaderValue::from_static("https://example.com"),
        );
        let mut origins = HashSet::new();
        origins.insert("https://example.com".to_string());
        match check_auth(&headers, "test_secret_12345", &origins) {
            SecurityCheckResult::Allowed { token_in_sec_ws } => assert!(token_in_sec_ws),
            SecurityCheckResult::Denied(_) => panic!("Expected allowed"),
        }
    }

    #[test]
    fn denies_websocket_protocol_with_forbidden_origin() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "sec-websocket-protocol",
            HeaderValue::from_static("minecraft-v1, test_secret_12345"),
        );
        headers.insert(
            axum::http::header::ORIGIN,
            HeaderValue::from_static("https://evil.com"),
        );
        let mut origins = HashSet::new();
        origins.insert("https://example.com".to_string());
        match check_auth(&headers, "test_secret_12345", &origins) {
            SecurityCheckResult::Denied(reason) => assert_eq!(reason, "Origin Not Allowed"),
            SecurityCheckResult::Allowed { .. } => panic!("Expected denied"),
        }
    }
}
