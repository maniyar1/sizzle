use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::convert::Infallible;

use warp::Filter;

use serde::{Deserialize, Serialize};

mod db;
mod html;

#[derive(Serialize, Deserialize)]
pub struct CommentIDs {
    ids: Vec<i64>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
// 'Everything is a Post', and this is the struct
pub struct Post {
    title: String,
    description: String,
    id: Option<i64>,
    parent: Option<i64>,
    comments: Option<Vec<i64>>,
}

// The main function and following are mostly warp-specific wrappers.
#[tokio::main]
async fn main() {
    println!("Creating sql pool");
    let pool = db::create_db_if_not_exist().await;
    let new_post = warp::path("new-post").and(warp::get()).and_then(new_post);

    let submit_pool = pool.clone();
    let submit = warp::path("submit")
        .and(warp::post())
        .and(warp::body::form())
        .and_then(move |params: HashMap<String, String>| submit(params, submit_pool.clone()));

    println!("setting paths");
    let home_pool = pool.clone();
    let home = warp::get().and_then(move || home(home_pool.clone()));
    let post = warp::path!("post" / i64).and_then(move |a: i64| view_post(a, pool.clone()));
    let routes = new_post.or(submit).or(post).or(home);

    println!("Running");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn home(pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    let html = html::home(pool).await;
    Ok(warp::reply::html(html))
}
async fn new_post() -> Result<impl warp::Reply, Infallible> {
    let html = html::new_post().await;
    Ok(warp::reply::html(html))
}

async fn submit(
    form: HashMap<String, String>,
    pool: SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    let parent = form["parent"].parse::<i64>();
    let parent = match parent {
        Ok(parent_id) => Some(parent_id),
        Err(_) => None,
    };
    let post = Post {
        title: form["title"].clone(),
        description: form["description"].clone(),
        parent,
        id: None,
        comments: None,
    };
    db::submit_post(post, parent, pool.clone()).await;
    home(pool.clone()).await
}

async fn view_post(id: i64, pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    let html = html::view_post(id, pool).await;
    Ok(warp::reply::html(html))
}
