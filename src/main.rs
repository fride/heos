#[macro_use]
extern crate serde_derive;

use actix_files as fs;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;

use pretty_env_logger::env_logger;

use askama::Template;
use rusty_heos::{HeosDriver, HeosResult};
mod ui;

mod templates;

#[get("/")]
async fn index(data: Data<Mutex<HeosDriver>>) -> String {
    let data = data.lock().unwrap();
    let zones = data.zones();
    let str = serde_json::to_string_pretty(&zones).unwrap();
    format!("{}", str)
}

#[get("/zones")]
async fn zones_route(data: Data<Mutex<HeosDriver>>) -> actix_web::Result<HttpResponse> {
    let data = data.lock().unwrap();
    let zones = data.zones();
    let template = templates::ZonesTemplate { zones };
    println!("Zones!!!");
    let s = template.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> crate::HeosResult<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let connection = rusty_heos::connect(Some("192.168.178.35:1255")).await?;
    let driver = rusty_heos::create_driver(connection).await?;
    driver.init().await;

    let data = Data::new(Mutex::new(driver));
    // we don't care about this right now ;)
    let _ = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            // .data_factory(||{
            //     rusty_heos::create_api()
            // })
            .service(index)
            .service(echo)
            .service(zones_route)
            .service(
                fs::Files::new("/static", "static")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .wrap(Logger::new("%a %{User-Agent}i"))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;

    Ok(())
}
