use chrono::Utc;
use rusqlite::{Connection, Error};
use serde_json::{json, Value};

pub async fn list_authors() -> Result<impl warp::Reply, Error> {
    println!("List");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut stmt = conn.prepare("SELECT id, name, created_at, updated_at FROM authors")?;
            let rows = stmt.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<usize, i16>(0)?,
                    "name": row.get::<usize, String>(1)?,
                    "created_at": row.get::<usize, String>(2)?,
                    "updated_at": row.get::<usize, String>(3)?,
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
pub async fn get_author(id: i16) -> Result<impl warp::Reply, Error> {
    println!("Get");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut stmt =
                conn.prepare("SELECT id, name, created_at, updated_at FROM authors WHERE id=?1")?;
            let rows = stmt.query_map([id], |row| {
                Ok(json!({
                    "id": row.get::<usize, i16>(0)?,
                    "name": row.get::<usize, String>(1)?,
                    "created_at": row.get::<usize, String>(2)?,
                    "updated_at": row.get::<usize, String>(3)?,
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

pub async fn create_author(name: &String) -> Result<impl warp::Reply, Error> {
    println!("Create");
    match Connection::open("/home/studentas/Documents/blog.db") {
        Ok(conn) => {
            let mut stmt = conn.prepare(
                "INSERT INTO authors (name, created_at, updated_at) VALUES (?1, ?2, ?3)",
            )?;
            let time_now = Utc::now();
            let now_str = time_now.format("%Y-%m-%d %H:%M:%S").to_string();
            stmt.execute([name, &now_str, &now_str])?;
            Ok(warp::reply::json(&format!(
                "User created with name: {}",
                name
            )))
        }
        Err(e) => Err(e),
    }
}
