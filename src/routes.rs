use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest};
use actix_web::cookie;

use sha2::{Sha256, Digest};

use sqlx::{query, Row};

use futures::join;

use crate::utils::{self, *};

#[post("/msg")]
async fn get_msg(data: web::Data<AppData>, to_name: web::Json<ToName>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");
    if !utils::validate(&data.pool, &session).await {
        return HttpResponse::NotFound()
            .body("");
    }

    let from_name = utils::get_name(&data.pool, &session)
        .await;
    if let Err(_) = from_name {
        return HttpResponse::NotFound()
            .body("")
    }

    let messages: Vec<SQLMessage> = sqlx::query_as::<_, SQLMessage>("select * from messages where (from_name = ? and to_name = ?) or (from_name = ? and to_name = ?)")
        .bind(&to_name.to_name)
        .bind(from_name.as_ref().unwrap())
        .bind(from_name.as_ref().unwrap())
        .bind(&to_name.to_name)
        .fetch_all(&data.pool)
        .await
        .unwrap();

    HttpResponse::Ok()
        .json(messages)
}

#[post("/add-msg")]
async fn add_msg(data: web::Data<AppData>, message: web::Json<Message>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");
    let name = utils::get_name(&data.pool, &session)
        .await;
    match name {
        Err(_) => return HttpResponse::NotFound()
            .body(""),
        _ => ()
    }

    if message.to_name == "" {
        return HttpResponse::NotFound()
            .body("")
    }

    let _ = sqlx::query("insert into messages values (?, ?, ?)")
        .bind(name.unwrap())
        .bind(&message.to_name)
        .bind(&message.msg)
        .execute(&data.pool)
        .await;

    HttpResponse::Ok()
        .body("Message added")
}

#[get("/get-chats")]
async fn get_chats(data: web::Data<AppData>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");
    let name = utils::get_name(&data.pool, &session)
        .await;
    if let Err(_) = name {
        return HttpResponse::NotFound()
            .body("")
    }

    let chats: Vec<ToName> = sqlx::query_as::<_, ToName>("select to_name from chats where from_name = ?")
        .bind(name.unwrap())
        .fetch_all(&data.pool)
        .await
        .unwrap();

    HttpResponse::Ok()
        .json(chats)
}

#[post("/add-chat")]
async fn add_chat(data: web::Data<AppData>, chat: web::Json<ToName>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");
    let name = utils::get_name(&data.pool, &session)
        .await;
    if let Err(_) = name {
        return HttpResponse::NotFound()
            .body("")
    }

    if chat.to_name.is_empty() {
        return HttpResponse::NotFound()
            .body("")
    }

    let query_chat = sqlx::query_as::<_, Chat>("select * from chats where from_name = ? and to_name = ?")
        .bind(name.as_ref().unwrap())
        .bind(&chat.to_name)
        .fetch_one(&data.pool)
        .await;

    if let Ok(_) = query_chat {
        return HttpResponse::NotFound()
            .body("")
    }

    let query_1 = sqlx::query("insert into chats values (?, ?)")
        .bind(name.as_ref().unwrap())
        .bind(&chat.to_name)
        .execute(&data.pool);
    let query_2 = sqlx::query("insert into chats values (?, ?)")
        .bind(&chat.to_name)
        .bind(name.as_ref().unwrap())
        .execute(&data.pool);
    let _ = join!(query_1, query_2);

    HttpResponse::Ok()
        .body("")
}

#[post("/login")]
async fn login(data: web::Data<AppData>, account: web::Json<Account>) -> impl Responder {
    let password_hash = sqlx::query("select password from users where name=?")
        .bind(&account.name)
        .fetch_one(&data.pool)
        .await;

    if let Err(_) = password_hash {
        return HttpResponse::NotFound()
            .await;
    }

    let password_hash: String = password_hash.unwrap()
        .get("password");

    let mut hasher = Sha256::new();
    hasher.update(account.password.as_bytes());
    let result = hex::encode(hasher.finalize());

    if result == password_hash {
        let id = utils::gen_session_id();
        let session = cookie::Cookie::new("session", &id);

        let _ = sqlx::query("insert into sessions values (?, ?)")
            .bind(&account.name)
            .bind(&id)
            .execute(&data.pool)
            .await;

        HttpResponse::Ok()
            .cookie(session)
            .await
    }else {
        HttpResponse::NotFound()
            .await
    }
}

#[post("/signup")]
async fn signup(data: web::Data<AppData>, account: web::Json<Account>) -> impl Responder {
    if utils::check_account(&data.pool, account.name.as_str()).await {
        return HttpResponse::NotFound()
            .body("");
    }

    let _ = sqlx::query("insert into users values (?, sha2(?, 256))")
        .bind(&account.name)
        .bind(&account.password)
        .execute(&data.pool)
        .await;

    HttpResponse::Ok()
        .body("")
}

#[get("/signout")]
async fn signout(data: web::Data<AppData>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");
    if !utils::validate(&data.pool, &session).await {
        return HttpResponse::NotFound()
            .body("");
    }

    let _ = sqlx::query("delete from sessions where name=?")
        .bind(utils::get_name(&data.pool, &session).await.unwrap())
        .execute(&data.pool)
        .await;

    HttpResponse::Ok()
        .body("")
}

#[get("/get-users")]
async fn get_users(data: web::Data<AppData>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");
    let name = utils::get_name(&data.pool, &session)
        .await;
    if let Err(_) = name {
        return HttpResponse::NotFound()
            .body("");
    }

    let users = sqlx::query_as::<_, User>("select name from users where not name = ?")
        .bind(name.unwrap())
        .fetch_all(&data.pool)
        .await
        .unwrap();

    HttpResponse::Ok()
        .json(users)
}

#[get("/validate")]
async fn validate(data: web::Data<AppData>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");

    if utils::validate(&data.pool, &session).await {
        HttpResponse::Ok()
    }else {
        HttpResponse::NotFound()
    }
}

#[get("/getname")]
async fn get_name(data: web::Data<AppData>, request: HttpRequest) -> impl Responder {
    let session = request.cookie("session");

     let name = utils::get_name(&data.pool, &session)
         .await;

     if let Err(_) = name {
         return HttpResponse::NotFound()
             .body("");
     }

     HttpResponse::Ok()
         .body(name.unwrap())
}
