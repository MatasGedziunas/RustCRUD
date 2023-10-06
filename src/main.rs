use std::convert::Infallible;

use middleware::MyError;
use validations::{Errors, TooShortError};
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::{reject::Reject, Filter};

use crate::hashing::hash_string;

mod classes {
    pub mod author;
    pub mod post;
}
mod author_db_managment;
mod crud;
mod hashing;
mod middleware;
mod post_db_managment;
mod user_db_managment;
mod validations;

#[tokio::main]
async fn main() {
    let author_api = crud::author_filter();
    println!("start web-server");
    let post_api = crud::post_filter();
    let user_api = crud::user_filter();
    let routes = author_api.or(post_api).or(user_api);
    // .recover(handle_rejection);
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl warp::Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else if err.find::<Errors>().is_some() {
        match err.find::<Errors>().unwrap() {
            Errors::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database Error".to_string(),
            ),
            Errors::InvalidJson => (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()),
            Errors::InvalidFileType => (StatusCode::BAD_REQUEST, "Invalid File Type".to_string()),
            Errors::InvalidBase64String => {
                (StatusCode::BAD_REQUEST, "Invalid Base64 String".to_string())
            }
            Errors::FileNotFound => (StatusCode::BAD_REQUEST, "File not found".to_string()),
            Errors::InvalidQuery => (StatusCode::BAD_REQUEST, "Invalid query".to_string()),
        }
    } else if err.find::<MyError>().is_some() {
        match err.find::<MyError>().unwrap() {
            MyError::JWTTokenError => (StatusCode::UNAUTHORIZED, "JWT Token Error".to_string()),
            MyError::JWTTokenCreationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "JWT Token Creation Error".to_string(),
            ),
            MyError::NoAuthHeaderError => (
                StatusCode::UNAUTHORIZED,
                "No Authorization Header".to_string(),
            ),
            MyError::InvalidAuthHeaderError => (
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization Header".to_string(),
            ),
            MyError::FailAuth => (
                StatusCode::UNAUTHORIZED,
                "Failed Authentication".to_string(),
            ),
            MyError::InvalidJson => (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()),
            // Add more custom error cases as needed
        }
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };
    Ok(warp::reply::with_status(message, code))
}
