use actix_4_jwt_auth::biscuit::jwa::SignatureAlgorithm;
use actix_4_jwt_auth::biscuit::jws::{RegisteredHeader, Secret};
use actix_4_jwt_auth::biscuit::{ClaimsSet, RegisteredClaims, JWT};
use serde_json::{json, Value};
use std::string::ToString;

fn get_secret() -> Secret {
    Secret::rsa_keypair_from_file("tests/harness/jwk_private_key.der").unwrap()
}

pub(crate) fn create_token(tokenize: Value) -> String {
    let signing_secret = get_secret();
    let decoded_token = JWT::new_decoded(
        From::from(RegisteredHeader {
            algorithm: SignatureAlgorithm::RS256,
            key_id: Some("53047d50-d12d-4de6-8b63-f75732a8b302".to_string()),
            ..Default::default()
        }),
        ClaimsSet::<Value> {
            registered: RegisteredClaims {
                issuer: None,
                subject: None,
                audience: None,
                not_before: None,
                expiry: None,
                id: None,
                issued_at: None,
            },
            private: tokenize,
        },
    );
    decoded_token
        .encode(&signing_secret)
        .unwrap()
        .unwrap_encoded()
        .to_string()
}

pub const JWT_AUTHORITY: &str = "my-api";

pub(crate) fn create_jwt_token() -> String {
    let claims = json!({
    "iss": format!("{}/", JWT_AUTHORITY),
    "sub": "CgVhZG1pbhIFbG9jYWw",
    "aud": [JWT_AUTHORITY],
    });
    create_token(claims)
}
