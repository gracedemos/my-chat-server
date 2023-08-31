mod utils;
mod routes;

use actix_web::{web, App, HttpServer};

use std::io;

use utils::AppData;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let data = web::Data::new(AppData::new().await);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(routes::get_msg)
            .service(routes::add_msg)
            .service(routes::login)
            .service(routes::signup)
            .service(routes::signout)
            .service(routes::validate)
            .service(routes::get_name)
    })
    .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
