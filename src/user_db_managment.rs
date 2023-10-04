use crate::{hashing, middleware::MyError, validations::NoMatchingUser};
use chrono::Utc;
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use jwt::*;
use rusqlite::{Connection, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::BTreeMap;

const JWT_SECRET: &[u8] = b"secret";

pub async fn create_user(username: &String, password: &String) -> Result<impl warp::Reply, Error> {
    println!("Create");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let salt = hashing::generate_salt();
            let hashed_password = hashing::hash_string(password, &salt);
            let hashed_username = hashing::hash_string(username, &salt);
            let mut stmt =
                conn.prepare("INSERT INTO users (username, password, salt) VALUES (?1, ?2, ?3)")?;
            stmt.execute([hashed_username, hashed_password, salt])?;
            Ok((warp::reply::json(&format!("User created with username: {}", username))))
        }
        Err(e) => Err(e),
    }
}

pub async fn login_user(
    username: &String,
    password: &String,
) -> Result<(impl warp::Reply, String), Error> {
    println!("Login");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut stmt = conn.prepare("SELECT id, Username, Password, Salt FROM users")?;
            let mut rows = stmt.query_map([], |row| {
                let db_id: i16 = row.get(0)?;
                let db_username: String = row.get(1)?;
                let db_password: String = row.get(2)?;
                let salt: String = row.get(3)?;
                if hashing::verify_string(&db_password, password, &salt)
                    && hashing::verify_string(&db_username, username, &salt)
                {
                    let token = generate_jwt_token(&db_id.to_string()).unwrap(); // Generate JWT token
                    return Ok((
                        json!({
                            "Username": &db_username,
                            "Password": &db_password,
                            "Salt": &salt,
                            "Valid": true,
                            "Token": token, // Include JWT token in response
                        }),
                        token,
                    ));
                }
                Ok((
                    json!({
                        "Username": &db_username,
                        "Password": &db_password,
                        "Salt": &salt,
                        "Valid": false,
                    }),
                    String::new(),
                ))
            })?;

            for row in rows {
                let (response, token) = row.unwrap();
                if response["Valid"] == true {
                    return Ok((warp::reply::json(&response), token));
                }
            }

            Err(rusqlite::Error::InvalidQuery)
        }
        Err(e) => Err(e),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

fn generate_jwt_token(id: &str) -> Result<String, MyError> {
    let my_claims = Claims {
        sub: id.to_owned(),
        exp: Utc::now()
            .checked_add_signed(chrono::Duration::minutes(20))
            .expect("Valid timestamp")
            .timestamp() as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &my_claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| MyError::JWTTokenCreationError)
}
