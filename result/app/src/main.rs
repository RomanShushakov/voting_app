use actix_web::{HttpServer, App, web, middleware};
use actix_files::Files;
use actix::*;

mod server;
mod session;
mod models;


#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let bind = "0.0.0.0:8080";
    println!("Starting server at: {}", &bind);

    let server = server::WebsocketServer::default().start();
    HttpServer::new(move ||
        {
            App::new()
                .data(server.clone())
                .wrap(middleware::Logger::default())
                .service(web::resource("/ws/").to(session::start_ws))
                .service(Files::new("", "./web_layout").index_file("index.html"))
        })
    .bind(&bind)?
    .run()
    .await
}
