use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


pub struct Author{
    id: i16, 
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

