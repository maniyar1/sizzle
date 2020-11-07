use crate::Post;

use sqlx::sqlite::{SqlitePool, SqliteQueryAs};

pub async fn submit_post(post: Post, parentid: Option<i64>, pool: SqlitePool) {
    let conn = pool.acquire().await.unwrap();
    if post.id != None {
        println!("Warning: Id exists when it shouldn't");
    }
    let post_clone = post.clone();
    sqlx::query(
        "
    INSERT INTO submissions ( title, description, parent, id ) VALUES ( ?, ?, ?, (SELECT Max(ID) FROM submissions) + 1);
        ",
    )
    .bind(post.title)
    .bind(post.description)
    .bind(parentid)
    .execute(conn)
    .await
    .unwrap();

    if parentid.is_some() {
        let parentid = parentid.unwrap();
        let id = get_post_id(post_clone.title, pool.clone()).await.unwrap();
        let parent = get_post(parentid, pool.clone()).await.unwrap();
        let mut new_comments: Vec<i64> = Vec::new();
        if parent.comments.is_some() {
            new_comments = parent.comments.unwrap();
        }
        new_comments.push(id);
        update_comments(parentid, new_comments, pool.clone()).await;
    }
}

pub async fn update_comments(id: i64, comments: Vec<i64>, pool: SqlitePool) {
    let conn = pool.acquire().await.unwrap();
    let comments_string: String = serde_json::to_string(&comments).unwrap();
    println!("Updating comments to {}", comments_string);
    sqlx::query(
        "
    UPDATE submissions SET children = ? WHERE id = ?
        ",
    )
    .bind(comments_string)
    .bind(id)
    .execute(conn)
    .await
    .unwrap();
}

pub async fn create_db_if_not_exist() -> SqlitePool {
    let pool: SqlitePool = SqlitePool::new("sqlite:submissions.db").await.unwrap();
    let conn = pool.acquire().await.unwrap();
    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS SUBMISSIONS(
    TITLE TEXT NOT NULL UNIQUE,
    DESCRIPTION TEXT NOT NULL,
    CHILDREN TEXT,
    PARENT INTEGER,
    ID INTEGER NOT NULL,
    UNIQUE(ID)
);
    INSERT OR REPLACE INTO submissions ( title, description, id ) VALUES ( \"\", \"\", 0);
",
    )
    .execute(conn)
    .await
    .unwrap();
    pool
}
pub async fn get_post(id: i64, pool: SqlitePool) -> Option<Post> {
    let mut conn = pool.acquire().await.unwrap();

    let row: (String, String, Option<String>, Option<i64>) =
        sqlx::query_as("SELECT title, description, children, parent FROM submissions WHERE id = ?")
            .bind(id)
            .fetch_one(&mut conn)
            .await
            .unwrap();
    println!("{:#?}", row);
    let mut comments: Option<Vec<i64>> = None;
    if row.2.is_some() {
        let comment_ids: Vec<i64> = serde_json::from_str(&row.2.unwrap()).unwrap();
        comments = Some(comment_ids);
    }
    Some(Post {
        title: row.0,
        description: row.1,
        id: Some(id),
        parent: row.3,
        comments,
    })
}

pub async fn get_post_id(title: String, pool: SqlitePool) -> Option<i64> {
    let mut conn = pool.acquire().await.unwrap();

    let row: (String, String, i64) =
        sqlx::query_as("SELECT title, description, id FROM submissions WHERE title = ?")
            .bind(title)
            .fetch_one(&mut conn)
            .await
            .unwrap();
    Some(row.2)
}

pub async fn get_posts_sorted_by_id(
    pool: SqlitePool,
) -> Option<Vec<(String, String, Option<i64>, i64)>> {
    let mut conn = pool.acquire().await.unwrap();

    let rows: Vec<(String, String, Option<i64>, i64)> =
        sqlx::query_as("SELECT title, description, parent, id FROM submissions ORDER BY id DESC")
            .fetch_all(&mut conn)
            .await
            .unwrap();
    Some(rows)
}
