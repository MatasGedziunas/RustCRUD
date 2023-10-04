use chrono::Utc;
use mime_guess::from_path;
use rusqlite::{Connection, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::vec;
use warp::reject::Rejection;

use crate::middleware::MyError;
use crate::validations::{DatabaseError, Errors};

pub async fn list_posts() -> Result<impl warp::Reply, Error> {
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            println!("Lista");
            let mut stmt = conn
                .prepare("SELECT id, title, body, author_id, created_at, updated_at FROM posts")?;
            let rows = stmt.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<usize, i16>(0)?,
                    "title": row.get::<usize, String>(1)?,
                    "body": row.get::<usize, String>(2)?,
                    "author_id": row.get::<usize, i16>(3)?,
                    "created_at": row.get::<usize, String>(4)?,
                    "updated_at": row.get::<usize, String>(5)?,
                }))
            })?;
            let mut v: Vec<Value> = Vec::new();
            for row in rows {
                v.push(row.unwrap());
            }

            Ok(warp::reply::json(&v))
        }
        Err(e) => Err(e),
    }
}
pub async fn get_post(id: i16) -> Result<impl warp::Reply, Error> {
    println!("Get");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut stmt = conn.prepare(
                "SELECT id, title, body, author_id, created_at, updated_at FROM posts WHERE id=?1",
            )?;
            let rows = stmt.query_map([id], |row| {
                Ok(json!({
                    "id": row.get::<usize, i16>(0)?,
                    "title": row.get::<usize, String>(1)?,
                    "body": row.get::<usize, String>(2)?,
                    "author_id": row.get::<usize, i16>(3)?,
                    "created_at": row.get::<usize, String>(4)?,
                    "updated_at": row.get::<usize, String>(5)?,
                }))
            })?;
            let mut v: Vec<Value> = Vec::new();
            for row in rows {
                v.push(row.unwrap());
            }
            Ok(warp::reply::json(&v))
        }
        Err(e) => Err(e),
    }
}

pub async fn create_post(
    title: &String,
    body: &String,
    author_id: &String,
) -> Result<impl warp::Reply, Error> {
    println!("Create");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => match author_id_exists(&conn, author_id) {
            Some(count) if count == 1 => {
                let mut stmt = conn.prepare("INSERT INTO posts (title, body, author_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)")?;
                let time_now = Utc::now();
                let now_str = time_now.format("%Y-%m-%d %H:%M:%S").to_string();
                stmt.execute([&title, &body, &author_id, &now_str, &now_str])?;
                Ok(warp::reply::json(&format!(
                    "Post created with info: title: {} ; body: {} ; author_id: {}",
                    title, body, author_id
                )))
            }
            Some(_) => Err(Error::InvalidQuery),
            None => Err(Error::InvalidQuery),
        },
        Err(e) => Err(e),
    }
}

fn author_id_exists(conn: &Connection, author_id: &String) -> Option<i16> {
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM Authors WHERE id = ?1")
        .expect("Failed to prepare statement");
    let count: i16 = stmt.query_row(&[&author_id], |row| row.get(0)).unwrap_or(0);

    if count > 0 {
        Some(count)
    } else {
        None
    }
}

pub async fn delete_post(id: i16) -> Result<impl warp::Reply, Error> {
    println!("Delete");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut stmt = conn.prepare("DELETE FROM posts WHERE id=?1")?;
            stmt.execute([&id])?;
            Ok(warp::reply::json(&format!("Post deleted with id: {}", id)))
        }
        Err(e) => Err(e),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct FileData {
    name: String,
    data: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FormData {
    files: Vec<FileData>,
    post_id: i16, // Include other fields as needed
}
pub async fn upload_files(data: Value) -> Result<impl warp::Reply, Rejection> {
    // Deserialize the JSON data into your Rust struct
    let form_data: FormData = serde_json::from_value(data).map_err(|e| {
        eprintln!("Failed to parse JSON: {:?}", e);
        warp::reject::custom(MyError::InvalidJson)
    })?;
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            // Process the files and other fields
            let mut file_names: Vec<String> = Vec::new();
            let mut not_uploaded_file_names: Vec<String> = Vec::new();
            let sql_query = "INSERT INTO post_files (file, post_id, name) VALUES (?1, ?2, ?3)";
            let post_id = form_data.post_id;
            for file in form_data.files {
                let filename = file.name;
                if (is_allowed_file_type(&filename)) {
                    let base64_data = file.data;
                    file_names.push(filename.to_owned());
                    let mut stmt = conn
                        .prepare(sql_query)
                        .map_err(|_| Err::<MyError, _>(Errors::DatabaseError))
                        .unwrap();
                    let _ = stmt
                        .execute([base64_data, post_id.to_string(), filename])
                        .map_err(|_| Err::<MyError, _>(Errors::DatabaseError));
                } else {
                    not_uploaded_file_names.push(filename);
                }
            }
            // Access other fields as needed
            if not_uploaded_file_names.is_empty() {
                Ok(warp::reply::json(&format!(
                    "Uploaded files {:?} ; to post {}",
                    file_names, post_id
                )))
            } else if file_names.is_empty() {
                Ok(warp::reply::json(&format!(
                    "Unable to upload files (wrong extensions) {:?}",
                    not_uploaded_file_names
                )))
            } else {
                Ok(warp::reply::json(&format!(
                    "Uploaded files {:?} ; to post {} \n
                Unable to upload files (wrong extensions) {:?}",
                    file_names, post_id, not_uploaded_file_names
                )))
            }

            // Rest of your upload logic...
        }
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}

pub fn is_allowed_file_type(file_name: &str) -> bool {
    let allowed_extensions = ["pdf", "txt", "docx", "doc"];
    if let Some(extension) = file_name.split('.').last() {
        allowed_extensions.contains(&extension)
    } else {
        false
    }
}
