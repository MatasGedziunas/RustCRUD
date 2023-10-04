use chrono::Utc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use jwt::Claims;
use sha2::{Sha256, Sha384};
use std::collections::BTreeMap;
use warp::{
    filters::header::headers_cloned,
    http::header::{HeaderMap, HeaderValue},
    reject::{Reject, Rejection},
    Filter,
};

use crate::user_db_managment;

const TOKEN_HEADER: &str = "Jwt-Token";
const JWT_SECRET: &[u8] = b"secret";

#[derive(Debug)]
pub enum MyError {
    JWTTokenError,
    JWTTokenCreationError,
    NoAuthHeaderError,
    InvalidAuthHeaderError,
    // Add more custom error variants as needed
    FailAuth,
}
impl Reject for MyError {}

// async fn validate_jwt_token(headers: HeaderMap<HeaderValue>) -> Result<String, Rejection> {
//     println!("{:?}", headers);
//     return Ok("".to_lowercase());
// }

pub fn with_auth() -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
    let time_now = Utc::now().timestamp() as usize;
    warp::any()
        .and(warp::header::<String>(TOKEN_HEADER))
        .and_then(move |token: String| async move {
            let validation = Validation::new(Algorithm::HS512);
            match decode::<user_db_managment::Claims>(
                &token,
                &DecodingKey::from_secret(JWT_SECRET),
                &validation,
            ) {
                Ok(claims) => {
                    if claims.claims.exp < time_now {
                        return Err(warp::reject::custom(FailAuth));
                    } else {
                        return Ok::<(), warp::Rejection>(());
                    }
                }
                Err(_) => Err(warp::reject::custom(FailAuth)),
            }
        })
}

#[derive(Debug)]
pub struct FailAuth;
impl warp::reject::Reject for FailAuth {}
