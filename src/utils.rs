use actix_web::cookie::Cookie;
use rand::RngCore;
use serde::{Serialize, Deserialize};

use sqlx::{Pool, MySql, Row};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub to_name: String,
    pub msg: String
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct SQLMessage {
    pub from_name: String,
    pub to_name: String,
    pub msg: String
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub password: String
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ToName {
    pub to_name: String
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub name: String
}

#[derive(sqlx::FromRow)]
pub struct Chat {
    pub from_name: String,
    pub to_name: String
}

pub struct AppData {
    pub pool: Pool<MySql>
}

impl AppData {
    pub async fn new() -> Self {
        AppData {
            pool: Pool::connect("mysql://my-chat:testpassword@127.0.0.1/my_chat")
                .await
                .unwrap()
        }
    }
}

pub fn gen_session_id() -> String {
    let mut id = [0; 16];
    rand::thread_rng()
        .fill_bytes(&mut id);

    hex::encode(id)
}

pub async fn validate(pool: &Pool<MySql>, session: &Option<Cookie<'_>>) -> bool {
    if let None = session {
        return false;
    }
    let session = session.as_ref()
        .unwrap();
    let session = session.value();

    let ids = sqlx::query("select id from sessions")
        .fetch_all(pool)
        .await
        .unwrap();

    for id in ids {
        if id.get::<String, _>("id") == session {
            return true;
        }
    }

    false
}

pub async fn get_name(pool: &Pool<MySql>, session: &Option<Cookie<'_>>) -> Result<String, String> {
    if !validate(pool, session).await {
        return Err(String::from("Invalid session id"));
    }
    let session = session.as_ref()
        .unwrap();
    let session = session.value();

    let name: String = sqlx::query("select name from sessions where id=?")
        .bind(session)
        .fetch_one(pool)
        .await
        .unwrap()
        .get("name");

    Ok(name)
}

pub async fn check_account(pool: &Pool<MySql>, name: &str) -> bool {
    let acc_names = sqlx::query("select name from users")
        .fetch_all(pool)
        .await
        .unwrap();

    for acc_name in acc_names {
        if acc_name.get::<String, _>("name") == name {
            return true;
        }
    }

    false
}
