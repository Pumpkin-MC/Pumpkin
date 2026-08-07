use std::{
    fs::OpenOptions,
    io::{ErrorKind, Write},
    path::Path as FsPath,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose};
use pumpkin_util::jwt::Jwks;
use pumpkin_util::p384::{
    PublicKey,
    ecdsa::{
        Signature, SigningKey,
        signature::{Signer, Verifier},
    },
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey},
};
use serde_json::{Value, json};

pub fn load_or_create_identity_key(path: &FsPath) -> std::io::Result<Arc<SigningKey>> {
    loop {
        match std::fs::read(path) {
            Ok(bytes) => {
                let key = SigningKey::from_pkcs8_der(&bytes).map_err(|error| {
                    std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("invalid NetherNet identity key: {error}"),
                    )
                })?;
                return Ok(Arc::new(key));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let key = loop {
                    let bytes = rand::random::<[u8; 48]>();
                    if let Ok(key) = SigningKey::from_slice(&bytes) {
                        break key;
                    }
                };
                let document = key
                    .to_pkcs8_der()
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
                let mut options = OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600)
                };
                match options.open(path) {
                    Ok(mut file) => {
                        file.write_all(document.as_bytes())?;
                        file.sync_all()?;
                        return Ok(Arc::new(key));
                    }
                    Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error),
                }
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn verify_and_strip_identity(
    offer: &str,
    oidc_verifier: Option<&(String, Jwks)>,
) -> Result<(String, PublicKey), String> {
    let identity = offer
        .lines()
        .find_map(|line| line.strip_prefix("a=identity:"))
        .ok_or_else(|| "SDP offer is missing its identity assertion".to_string())?;
    let identity = general_purpose::STANDARD
        .decode(identity.trim())
        .map_err(|error| format!("invalid identity encoding: {error}"))?;
    let identity: Value = serde_json::from_slice(&identity)
        .map_err(|error| format!("invalid identity JSON: {error}"))?;
    if identity["idp"]["protocol"] != "default"
        || identity["idp"]["domain"].as_str().is_none_or(str::is_empty)
    {
        return Err("invalid identity provider".to_string());
    }
    let assertion = identity["assertion"]
        .as_str()
        .ok_or_else(|| "identity assertion is missing".to_string())?;
    let assertion: Value = serde_json::from_str(assertion)
        .map_err(|error| format!("invalid nested identity assertion: {error}"))?;
    let token = assertion["token"]
        .as_str()
        .ok_or_else(|| "identity token is missing".to_string())?;
    if let Some((issuer, keys)) = oidc_verifier {
        pumpkin_util::jwt::verify_oidc_token(token, issuer, keys)
            .map_err(|error| format!("invalid GameServerToken: {error}"))?;
    } else {
        validate_token_expiration(token)?;
    }
    let public_key = pumpkin_util::jwt::extract_cpk_from_token(token)
        .map_err(|error| format!("invalid identity public key: {error}"))?;
    let fingerprints = assertion["fingerprints"]
        .as_str()
        .ok_or_else(|| "fingerprint assertion is missing".to_string())?;
    verify_fingerprint_assertion(fingerprints, offer, &public_key)?;

    let mut stripped = offer
        .lines()
        .filter(|line| !line.starts_with("a=identity:"))
        .collect::<Vec<_>>()
        .join("\r\n");
    stripped.push_str("\r\n");
    Ok((stripped, public_key))
}

fn validate_token_expiration(token: &str) -> Result<(), String> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| "malformed identity token".to_string())?;
    let payload = general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| format!("invalid identity token payload: {error}"))?;
    let payload: Value = serde_json::from_slice(&payload)
        .map_err(|error| format!("invalid identity token claims: {error}"))?;
    let expiration = payload["exp"]
        .as_i64()
        .ok_or_else(|| "identity token has no expiration".to_string())?;
    if expiration < unix_time() {
        return Err("identity token has expired".to_string());
    }
    Ok(())
}

fn verify_fingerprint_assertion(
    assertion: &str,
    sdp: &str,
    public_key: &PublicKey,
) -> Result<(), String> {
    let mut parts = assertion.split('.');
    let header = parts.next().ok_or_else(|| "malformed JWS".to_string())?;
    let detached = parts.next().ok_or_else(|| "malformed JWS".to_string())?;
    let signature = parts.next().ok_or_else(|| "malformed JWS".to_string())?;
    if !detached.is_empty() || parts.next().is_some() {
        return Err("fingerprint assertion is not a detached JWS".to_string());
    }
    let header_json = general_purpose::URL_SAFE_NO_PAD
        .decode(header)
        .map_err(|error| format!("invalid fingerprint header: {error}"))?;
    let header_json: Value = serde_json::from_slice(&header_json)
        .map_err(|error| format!("invalid fingerprint header: {error}"))?;
    if header_json["alg"] != "ES384" {
        return Err("fingerprint assertion must use ES384".to_string());
    }
    let payload = fingerprint_payload(sdp)?;
    let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let signature = general_purpose::URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|error| format!("invalid fingerprint signature: {error}"))?;
    let signature = Signature::from_slice(&signature)
        .map_err(|error| format!("invalid ES384 signature: {error}"))?;
    let verifying_key = pumpkin_util::p384::ecdsa::VerifyingKey::from(public_key);
    verifying_key
        .verify(format!("{header}.{payload_b64}").as_bytes(), &signature)
        .map_err(|_| "fingerprint signature verification failed".to_string())
}

pub fn add_server_identity(sdp: &str, key: &SigningKey) -> Result<String, String> {
    let public_key = PublicKey::from(key.verifying_key());
    let public_der = public_key
        .to_public_key_der()
        .map_err(|error| error.to_string())?;
    let public_key = general_purpose::STANDARD.encode(public_der.as_bytes());
    let now = unix_time();
    let token = sign_jws(
        key,
        &json!({"alg": "ES384", "x5u": public_key}),
        &json!({"exp": now + 60, "iat": now, "cpk": public_key}),
    )?;
    let fingerprints = sign_detached(key, fingerprint_payload(sdp)?);
    let assertion = serde_json::to_string(&json!({
        "fingerprints": fingerprints,
        "token": token,
    }))
    .map_err(|error| error.to_string())?;
    let identity = json!({
        "assertion": assertion,
        "idp": {"domain": "self", "protocol": "default"},
    });
    let identity = general_purpose::STANDARD
        .encode(serde_json::to_vec(&identity).map_err(|error| error.to_string())?);

    let position = sdp
        .find("m=")
        .ok_or_else(|| "answer SDP has no media section".to_string())?;
    let mut answer = String::with_capacity(sdp.len() + identity.len() + 14);
    answer.push_str(&sdp[..position]);
    answer.push_str("a=identity:");
    answer.push_str(&identity);
    answer.push_str("\r\n");
    answer.push_str(&sdp[position..]);
    Ok(answer)
}

fn fingerprint_payload(sdp: &str) -> Result<Vec<u8>, String> {
    let fingerprints = sdp
        .lines()
        .filter_map(|line| line.strip_prefix("a=fingerprint:"))
        .map(|fingerprint| {
            let (algorithm, digest) = fingerprint
                .trim_end()
                .split_once(' ')
                .ok_or_else(|| "malformed DTLS fingerprint".to_string())?;
            Ok(json!({"algorithm": algorithm, "digest": digest}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if fingerprints.is_empty() {
        return Err("SDP has no DTLS fingerprint".to_string());
    }
    serde_json::to_vec(&json!({
        "fingerprint": fingerprints,
    }))
    .map_err(|error| error.to_string())
}

fn sign_jws(key: &SigningKey, header: &Value, payload: &Value) -> Result<String, String> {
    let header = general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(header).map_err(|error| error.to_string())?);
    let payload = general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(payload).map_err(|error| error.to_string())?);
    let input = format!("{header}.{payload}");
    let signature: Signature = key.sign(input.as_bytes());
    Ok(format!(
        "{input}.{}",
        general_purpose::URL_SAFE_NO_PAD.encode(signature.to_bytes())
    ))
}

fn sign_detached(key: &SigningKey, payload: Vec<u8>) -> String {
    let header = general_purpose::URL_SAFE_NO_PAD.encode(b"{\"alg\":\"ES384\"}");
    let payload = general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let signature: Signature = key.sign(format!("{header}.{payload}").as_bytes());
    format!(
        "{header}..{}",
        general_purpose::URL_SAFE_NO_PAD.encode(signature.to_bytes())
    )
}

fn unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_identity_is_valid_and_verifiable() {
        let key = SigningKey::from_slice(&[7; 48]).unwrap();
        let sdp = "v=0\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=fingerprint:sha-256 AA:BB\r\n";
        let answer = add_server_identity(sdp, &key).unwrap();
        let (_, public_key) = verify_and_strip_identity(&answer, None).unwrap();
        assert_eq!(public_key, PublicKey::from(key.verifying_key()));
    }

    #[test]
    fn fingerprint_payload_contains_every_sdp_fingerprint() {
        let sdp = "v=0\r\na=fingerprint:sha-256 AA:BB\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=fingerprint:sha-384 CC:DD\r\n";
        assert_eq!(
            fingerprint_payload(sdp).unwrap(),
            br#"{"fingerprint":[{"algorithm":"sha-256","digest":"AA:BB"},{"algorithm":"sha-384","digest":"CC:DD"}]}"#,
        );
    }

    #[test]
    fn configured_oidc_validation_rejects_untrusted_identity_tokens() {
        let key = SigningKey::from_slice(&[7; 48]).unwrap();
        let sdp = "v=0\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=fingerprint:sha-256 AA:BB\r\n";
        let answer = add_server_identity(sdp, &key).unwrap();
        let verifier = (
            "https://issuer.example".to_string(),
            Jwks { keys: Vec::new() },
        );
        assert!(verify_and_strip_identity(&answer, Some(&verifier)).is_err());
    }
}
