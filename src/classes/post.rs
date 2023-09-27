use chrono::{DateTime, Utc};
use crate::classes::author::Author;
pub struct Post{
    id: i16,
    title: String,
    body: String,
    author_id: Option<Author>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}