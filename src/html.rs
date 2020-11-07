use crate::db;
use sqlx::sqlite::SqlitePool;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use futures::future::{BoxFuture, FutureExt};

pub async fn home(pool: SqlitePool) -> String {
    let mut html = String::new();
    let posts = db::get_posts_sorted_by_id(pool).await.unwrap();
    for post in posts {
        if post.2.is_none() {
            html.push_str(&format!(
                "<a href=\"/post/{id}\"> {title} </p> <br>",
                title = &post.0,
                id = post.3
            ));
        }
    }
    let final_html = format!(
        "<!doctype html>
<html lang=\"en\">
    <head>
	<meta name=\"viewport\" content=\"width=device-width\">
        <link rel=icon href=/favicon.png>
        <meta charset=\"UTF-8\">
        <title>Piazza Clone</title>
    </head>
    <body>
        <main>
        <header> 
        <a href=\"/new-post\"> new post </a>
        </header>
         {}
        </main>
    </body>
</html>",
        html
    );
    final_html
}

pub async fn view_post(id: i64, pool: SqlitePool) -> String {
    let post = db::get_post(id, pool.clone()).await.unwrap();
    if post.comments.is_some() {
        let comments = post.comments.unwrap();
        for comment in comments {
            let comment_post = db::get_post(comment, pool.clone()).await.unwrap();
            println!("{:#?}", comment_post);
        }
    }
    let html = format!(
        "<!doctype html>
<html lang=\"en\">
    <head>
	<meta name=\"viewport\" content=\"width=device-width\">
        <link rel=icon href=/favicon.png>
        <meta charset=\"UTF-8\">
        <title>Piazza Clone</title>
    </head>
    <body>
        <main>
         {}
        </main>
    </body>
</html>",
        get_post_html(id, pool.clone(), 0).await
    );
    html
}
pub fn get_post_html(id: i64, pool: SqlitePool, layer: u8) -> BoxFuture<'static, String> {
    async move {
        let post = db::get_post(id, pool.clone()).await.unwrap();
        let mut html: String;
        if layer == 0 {
            html = format!(
                "
        <div style=\"margin-left: {indent}em\">
        <h1> {title} </h1>
        <p>
        {desc} </span> </p>",
                title = post.title,
                desc = post.description,
                indent = layer
            );
        } else if layer <= 6 {
            html = format!(
                "
        <div style=\"margin-left: {indent}em\">
        <h{layer}> {title} </h{layer}>
        <p>
        {desc} </span> </p>",
                title = post.title,
                desc = post.description,
                layer = layer,
                indent = layer
            );
        } else {
            html = format!(
                "
        <div style=\"margin-left: {indent}em\">
        <h6> {title} </h6>
        <p>
        {desc} </span> </p>",
                title = post.title,
                desc = post.description,
                indent = layer
            );
        }
        if post.comments.is_some() {
            let comments = post.comments.unwrap();
            for comment in comments {
                html = format!(
                    "{} \n {}",
                    html,
                    get_post_html(comment, pool.clone(), layer + 1).await
                );
            }
        }
        html
    }
    .boxed()
}

pub async fn new_post() -> String {
    let path = Path::new("new-post.html");
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open new-post.html {}", why),
        Ok(file) => file,
    };
    let mut html = String::new();
    file.read_to_string(&mut html).unwrap();
    html
}
