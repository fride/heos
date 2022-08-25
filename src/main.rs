#[macro_use]
extern crate serde_derive;

use actix_files as fs;

use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use actix_web::http::header::HeaderMap;

use pretty_env_logger::env_logger;

use askama::Template;
use tokio::time::sleep;


use rusty_heos::{Controller, HeosDriver, HeosResult};

mod templates;

fn is_json(headers: & HeaderMap) -> bool
{
    headers
        .get("Accept")
        .map_or(false, |header| {
            header.to_str().unwrap().contains("application/json")
        })
}

#[get("/")]
async fn index(data: Data<Mutex<HeosDriver>>) -> String {
    let data = data.lock().unwrap();
    let zones = data.zones();
    let str = serde_json::to_string_pretty(&zones).unwrap();
    format!("{}", str)
}

#[get("/zones")]
async fn zones_route(req: HttpRequest,data: Data<Mutex<HeosDriver>>) -> actix_web::Result<HttpResponse> {
    let data = data.lock().unwrap();
    let zones = data.zones();
    if is_json(req.headers()) {
        println!("sending json");
        let json = serde_json::to_string_pretty(&zones).unwrap();
        Ok(
            HttpResponse::Ok()
                .content_type("application/json")
                .body(json))
    } else {
        let template = templates::ZonesTemplate::new(zones);
        let s = template.render().unwrap();
        Ok(HttpResponse::Ok().content_type("text/html").body(s))
    }

}

#[get("/players")]
async fn all_players(
    req: HttpRequest,
    data: Data<Mutex<HeosDriver>>,
) -> actix_web::Result<HttpResponse> {
    println!("Got /players");
    let data = data.lock().unwrap();
    println!("Got /players 1");
    let players = data.players();
    println!("Got /players 2");
    let headers = req.headers();
    println!("Got /players 3");
    if headers
        .get("Accept")
        //TODO this is ugly!
        .map_or(false, |header| {
            header.to_str().unwrap().contains("application/json")
        })
    {
        let json = serde_json::to_string_pretty(&players).unwrap();
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json))
    } else {
        let json = serde_json::to_string_pretty(&players).unwrap();
        let html = format!("<pre>{}</pre>", &json);
        Ok(HttpResponse::Ok().content_type("text/html").body(html))
    }
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
    // let connection = rusty_heos::connect(Some("192.168.178.35:1255")).await?;
    let connection = rusty_heos::connect::<&str>(None).await?;
    let mut controller = Controller::new(connection).await?;

    controller.init().await;
    println!("controller: {:?}", &controller.get_players());
    println!("controller: {:?}", &controller.get_music_sources());

    //
    // let driver = rusty_heos::create_driver(connection).await?;
    // driver.init().await;
    //
    // println!("Starting web service");
    // let data = Data::new(Mutex::new(driver));
    // // we don't care about this right now ;)
    // let _ = HttpServer::new(move || {
    //     App::new()
    //         .app_data(data.clone())
    //         // .data_factory(||{
    //         //     rusty_heos::create_api()
    //         // })
    //         .service(index)
    //         .service(echo)
    //         .service(zones_route)
    //         .service(all_players)
    //         .service(
    //             fs::Files::new("/static", "static")
    //                 .show_files_listing()
    //                 .use_last_modified(true),
    //         )
    //         .wrap(Logger::new("%a %{User-Agent}i"))
    //         .route("/hey", web::get().to(manual_hello))
    // })
    // .bind(("127.0.0.1", 8080))?
    // .run()
    // .await;

    Ok(())
}
