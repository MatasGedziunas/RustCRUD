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

use crate::{post_db_managment::is_allowed_file_type, user_db_managment, validations::Errors};

const TOKEN_HEADER: &str = "Jwt-Token";
const FILE_HEADER: &str = "content-type";
const JWT_SECRET: &[u8] = b"secret";

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
pub fn check_file_type() -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::<String>(FILE_HEADER))
        .and_then(move |file_type: String| async move {
            match is_allowed_file_type(&file_type) {
                true => return Ok::<(), warp::Rejection>(()),
                false => return Err(warp::reject::custom(Errors::InvalidFileType)),
            }
        })
}

#[derive(Debug)]
pub struct FailAuth;
impl warp::reject::Reject for FailAuth {}

#[derive(Debug)]
pub enum MyError {
    JWTTokenError,
    JWTTokenCreationError,
    NoAuthHeaderError,
    InvalidAuthHeaderError,
    // Add more custom error variants as needed
    FailAuth,
    InvalidJson,
}
impl Reject for MyError {}
