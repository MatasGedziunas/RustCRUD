use base64::{engine, Engine};
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

const ALLOWED_FILE_SIZE: usize = 10 * 1024 * 1024;

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
    files: Value,
) -> Result<impl warp::Reply, Errors> {
    println!("Create");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => match author_id_exists(&conn, author_id) {
            Some(count) if count == 1 => {
                let time_now = Utc::now();
                let now_str = time_now.format("%Y-%m-%d %H:%M:%S").to_string();
                let post_id = match conn.execute(
                    "INSERT INTO posts (title, body, author_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    &[title, body, author_id, &now_str, &now_str],
                ) {
                    Ok(_) => conn.last_insert_rowid(), // Get the last inserted row id (post_id)
                    Err(_) => return Err(Errors::DatabaseError),
                };
                let form_data: FormData = serde_json::from_value(files).map_err(|e| {
                    eprintln!("Failed to parse JSON: {:?}", e);
                    Errors::InvalidJson
                })?;
                // Process the files and other fields
                let mut file_names: Vec<String> = Vec::new();
                let mut not_uploaded_file_names: Vec<String> = Vec::new();
                let sql_query = "INSERT INTO post_files (file, post_id, name) VALUES (?1, ?2, ?3)";
                for file in form_data.files {
                    let filename = file.name;
                    let base64_data = file.data;
                    match (validate(&filename, &base64_data)) {
                        Ok(true) => {
                            file_names.push(filename.to_owned());
                            let mut stmt = conn
                                .prepare(sql_query)
                                .map_err(|_| Err::<MyError, _>(Errors::DatabaseError))
                                .unwrap();
                            let _ = stmt
                                .execute([base64_data, post_id.to_string(), filename])
                                .map_err(|_| Err::<MyError, _>(Errors::DatabaseError));
                        }
                        Ok(false) => {
                            not_uploaded_file_names.push(filename);
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ok(warp::reply::json(&json!({
                    "post created with post_id": &post_id
                })))
            }
            Some(_) => Err(Errors::DatabaseError),
            None => Err(Errors::DatabaseError),
        },
        Err(_) => Err(Errors::DatabaseError),
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
    // post_id: String, // post_id: i16, // Include other fields as needed
}
pub async fn upload_files(
    data: Value,
    post_id: &String,
    conn: Option<Connection>,
) -> Result<impl warp::Reply, Rejection> {
    // Deserialize the JSON data into your Rust struct
    println!("upload");
    let conn = conn.unwrap_or_else(|| {
        Connection::open("/home/studentas/Documents/blog.db")
            .expect("Failed to open database connection")
    });
    let form_data: FormData = serde_json::from_value(data).map_err(|e| {
        eprintln!("Failed to parse JSON: {:?}", e);
        warp::reject::custom(MyError::InvalidJson)
    })?;
    // Process the files and other fields
    let mut file_names: Vec<String> = Vec::new();
    let mut not_uploaded_file_names: Vec<String> = Vec::new();
    let sql_query = "INSERT INTO post_files (file, post_id, name) VALUES (?1, ?2, ?3)";
    for file in form_data.files {
        let filename = file.name;
        let base64_data = file.data;
        match (validate(&filename, &base64_data)) {
            Ok(true) => {
                file_names.push(filename.to_owned());
                let mut stmt = conn
                    .prepare(sql_query)
                    .map_err(|_| Err::<MyError, _>(Errors::DatabaseError))
                    .unwrap();
                let _ = stmt
                    .execute([base64_data, post_id.to_string(), filename])
                    .map_err(|_| Err::<MyError, _>(Errors::DatabaseError));
            }
            Ok(false) => {
                not_uploaded_file_names.push(filename);
            }
            Err(e) => return Err(warp::reject::custom(e)),
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
            "Unable to upload files {:?}",
            not_uploaded_file_names
        )))
    } else {
        Ok(warp::reply::json(&format!(
            "Uploaded files {:?} ; to post {} \n
                Unable to upload files {:?}",
            file_names, post_id, not_uploaded_file_names
        )))
    }
}

pub async fn download_files(
    post_id: &String,
    file_name: &String,
) -> Result<impl warp::Reply, Rejection> {
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut stmt = conn
                .prepare("SELECT file FROM post_files WHERE post_id = ?1 AND name = ?2")
                .map_err(|_| warp::reject::custom(DatabaseError))?;
            let row = stmt
                .query_map([&post_id, &file_name], |row| {
                    let file_base64: String = row.get(0)?; // Assuming the "file" column is a String
                    let file_data = engine::general_purpose::STANDARD
                        .decode(file_base64)
                        .unwrap();
                    Ok(file_data)
                })
                .unwrap()
                .next();
            match row {
                Some(result) => {
                    match result {
                        Ok(file_data) => {
                            let response = warp::http::Response::builder()
                                .header("Content-Type", "application/octet-stream") // Set the appropriate content type
                                .header(
                                    "Content-Disposition",
                                    format!("attachment; filename=\"{}\"", file_name),
                                ) // Set the file name for download
                                .body(warp::hyper::Body::from(file_data))
                                .map_err(|_| warp::reject::custom(Errors::DatabaseError))?;
                            return Ok(response);
                        }
                        Err(_) => return Err(warp::reject::custom(Errors::DatabaseError)),
                    }
                }
                None => Err(warp::reject::custom(Errors::FileNotFound)),
            }
        }
        Err(_) => Err(warp::reject::custom(Errors::DatabaseError)),
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

fn get_base64_file_size(base64: &str) -> Result<usize, Errors> {
    match engine::general_purpose::STANDARD.decode(base64) {
        Ok(decoded_data) => {
            println!("File Size: {}", decoded_data.len());
            return Ok(decoded_data.len());
        }
        Err(_) => return Err(Errors::InvalidBase64String),
    }
}

fn validate(filename: &str, base64: &str) -> Result<bool, Errors> {
    if is_allowed_file_type(filename) {
        match get_base64_file_size(base64) {
            Ok(size) => {
                if size <= ALLOWED_FILE_SIZE {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(e), // Propagate the error up as Err
        }
    } else {
        Ok(false)
    }
}
