use chrono::Utc;
use mime_guess::from_path;
use rusqlite::{Connection, Error};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

pub async fn upload_file(filename: &str, post_id: i16) -> Result<impl warp::Reply, Error> {
    println!("Upload");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut filename_parts = filename.split(".");
            if let Some(first_part) = filename_parts.next() {
                // let mut stmt = "INSERT INTO "
                Ok(warp::reply::json(&"File uploaded sucessfully"))
            } else {
                // The iterator is empty
                Err(Error::InvalidQuery)
            }
        }
        Err(e) => Err(e),
    }
}

fn is_allowed_file_type(file_type: &str) -> bool {
    let allowed_types = vec![
        "application/pdf",
        "text/plain",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/msword",
    ];
    allowed_types.contains(&file_type)
}
