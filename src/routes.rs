use actix_web::{get, post, options, web, HttpResponse, Responder};

use crate::utils::{Message, Messages};

#[get("/")]
async fn get(data: web::Data<Messages>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(data)
}

#[post("/")]
async fn add(data: web::Data<Messages>, msg: web::Json<Message>) -> impl Responder {
    *data.message_count.lock()
        .unwrap() += 1;

    let mut messages = data.messages.lock()
        .unwrap();

    messages.push(msg.0);

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .body("Message added")
}

#[options("/")]
async fn options() -> impl Responder {
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Access-Control-Allow-Headers", "Content-Type"))
        .await
}
