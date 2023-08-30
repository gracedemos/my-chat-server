mod utils;
mod routes;

use actix_web::{web, App, HttpServer};

use std::io;

use utils::Messages;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let data = web::Data::new(Messages::new());

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(routes::get)
            .service(routes::add)
            .service(routes::options)
    })
    .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
