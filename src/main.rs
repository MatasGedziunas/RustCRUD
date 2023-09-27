use warp::Filter;

mod classes {
    pub mod author;
    pub mod post;
}
mod author_db_managment;
mod crud;
mod post_db_managment;
mod validations;

#[tokio::main]
async fn main() {
    let author_api = crud::author_filter();
    println!("start web-server");
    let post_api = crud::post_filter();
    warp::serve(author_api.or(post_api))
        .run(([127, 0, 0, 1], 8080))
        .await;
}
