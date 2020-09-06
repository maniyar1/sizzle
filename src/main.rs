use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use warp::Filter;

#[derive(Debug)]
struct Post {
    title: String,
    description: String,
}

#[tokio::main]
async fn main() {
    let pool: SqlitePool = SqlitePool::new("sqlite:todos.db").await.unwrap();
    let new_post_text = new_post().await;
    let new_post = warp::path("new-post")
        .and(warp::get())
        .map(move || warp::reply::html(new_post_text.clone()));

    let submit = warp::path("submit")
        .and(warp::post())
        .and(warp::body::form())
        .and_then(move |params: HashMap<String, String>| submit(params, pool.clone()));

    let home = warp::path("home").and(warp::get()).and_then(home);
    let _error = warp::any().map(|| "404, page not found");
    let routes = new_post.or(home).or(submit);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn new_post() -> String {
    let path = Path::new("new-post.html");
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let mut new_post = String::new();
    match file.read_to_string(&mut new_post) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{} contains:\n{}", display, new_post),
    }
    new_post
}

async fn home() -> Result<impl warp::Reply, Infallible> {
    let path = Path::new("home.html");
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let mut home = String::new();
    match file.read_to_string(&mut home) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{} contains:\n{}", display, home),
    }
    Ok(home)
}

async fn submit(
    form: HashMap<String, String>,
    pool: SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let description = &form["description"];
    let title = &form["title"];

    let mut conn = pool.acquire().await.unwrap();
    sqlx::query(
        "
INSERT INTO submissions ( description ) VALUES ( ? );
        ",
    )
    .bind(description)
    .execute(&mut conn)
    .await
    .unwrap();
    Ok(warp::reply::reply())
}

mod db {}
