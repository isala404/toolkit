use chrono::{NaiveDateTime, Utc};
use cron_parser::parse;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub aud: String,
    pub user_id: String,
}

pub fn extract_claims(token: Option<&str>) -> Result<Claims, String> {
    let token = match token {
        Some(token) => token,
        None => {
            return Err("unable to extract token".to_string());
        }
    };

    let token = match token.strip_prefix("Bearer ") {
        Some(token) => token,
        None => {
            return Err("invalid token".to_string());
        }
    };

    // extract user id from token
    let key = DecodingKey::from_secret(&[]);
    let mut validation = Validation::new(Algorithm::HS256);
    validation.insecure_disable_signature_validation();

    match decode::<Claims>(token, &key, &validation) {
        Ok(data) => Ok(data.claims), // Return the data in the Ok variant
        Err(_) => Err("invalid token".to_string()),
    }
}

pub fn decode_cron(cron_pattern: &str) -> Result<NaiveDateTime, String> {
    let next = std::panic::catch_unwind(|| parse(cron_pattern, &Utc::now()));

    let next = match next {
        Ok(next) => next,
        Err(_) => {
            return Err("Invalid cron pattern".to_string());
        }
    };
    let next = match next {
        Ok(next) => next,
        Err(_) => {
            return Err("Invalid cron pattern".to_string());
        }
    };
    Ok(next.naive_utc())
}
