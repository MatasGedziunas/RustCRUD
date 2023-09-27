use chrono::Utc;
use rusqlite::{Connection, Error};

use crate::hashing;

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

pub async fn login_user(username: &String, password: &String) -> Result<impl warp::Reply, Error> {}
