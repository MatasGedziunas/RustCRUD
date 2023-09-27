use std::collections::HashMap;
use warp::{reject::reject, Filter};

use crate::validations::{DatabaseError, MissingParameter};
use crate::{author_db_managment, post_db_managment, validations::Validations};
pub fn author_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let author_base = warp::path("authors");

    let get = warp::get()
        .and(author_base)
        .and(warp::path::param::<i16>())
        .and(warp::path::end())
        .and_then(get_author);

    let create = warp::post()
        .and(author_base)
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::path::end())
        .and_then(create_author);

    let list = warp::get()
        .and(author_base)
        .and(warp::path::end())
        .and_then(list_authors);

    create.or(get).or(list)
}

pub async fn list_authors() -> Result<impl warp::Reply, warp::Rejection> {
    match author_db_managment::list_authors().await {
        Ok(reply) => Ok(reply),
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}

pub async fn get_author(id: i16) -> Result<impl warp::Reply, warp::Rejection> {
    match author_db_managment::get_author(id).await {
        Ok(reply) => Ok(reply),
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}

pub async fn create_author(
    param: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(name) = param.get("name") {
        match Validations::validate(name) {
            Ok(_) => match author_db_managment::create_author(name).await {
                Ok(reply) => Ok(reply),
                Err(_) => Err(warp::reject::custom(DatabaseError)),
            },
            Err(rejection) => Err(rejection),
        }
    } else {
        // Handle the case where "name" is missing in the HashMap.
        Err(warp::reject::custom(MissingParameter))
    }
}

pub fn post_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let post_base = warp::path("posts");

    let get = warp::get()
        .and(post_base)
        .and(warp::path::param::<i16>())
        .and(warp::path::end())
        .and_then(get_post);

    let list = warp::get()
        .and(post_base)
        .and(warp::path::end())
        .and_then(list_posts);

    let create = warp::post()
        .and(post_base)
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::path::end())
        .and_then(create_post);

    let delete = warp::post()
        .and(post_base)
        .and(warp::path("delete"))
        .and(warp::path::param::<i16>())
        .and(warp::path::end())
        .and_then(delete_post);

    create.or(get).or(list).or(delete)
}

async fn list_posts() -> Result<impl warp::Reply, warp::Rejection> {
    match post_db_managment::list_posts().await {
        Ok(reply) => Ok(reply),
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}

async fn get_post(id: i16) -> Result<impl warp::Reply, warp::Rejection> {
    match post_db_managment::get_post(id).await {
        Ok(reply) => Ok(reply),
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}

async fn create_post(param: HashMap<String, String>) -> Result<impl warp::Reply, warp::Rejection> {
    if let (Some(title), Some(body), Some(author_id)) = (
        param.get("title"),
        param.get("body"),
        param.get("author_id"),
    ) {
        match Validations::validate(title) {
            Ok(_) => match post_db_managment::create_post(title, body, author_id).await {
                Ok(reply) => Ok(reply),
                Err(_) => Err(reject()),
            },
            Err(rejection) => Err(rejection),
        }
    } else {
        Err(warp::reject::custom(DatabaseError))
    }
}

async fn delete_post(id: i16) -> Result<impl warp::Reply, warp::Rejection> {
    match post_db_managment::delete_post(id).await {
        Ok(reply) => Ok(reply),
        Err(_) => Err(warp::reject::custom(DatabaseError)),
    }
}
