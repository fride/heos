#[macro_use]
extern crate serde_derive;

use std::sync::Mutex;

use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use pretty_env_logger::env_logger;

use rusty_heos::{HeosDriver, HeosResult};

mod ui;

#[get("/")]
async fn index(data: Data<Mutex<HeosDriver>>) -> String {
    let data = data.lock().unwrap();
    let zones = data.zones();
    let str = serde_json::to_string_pretty(&zones).unwrap();
    format!("{}", str)
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

    // use std::{thread, time};
    //
    // let ten_millis = time::Duration::from_secs(10);
    // let _now = time::Instant::now();
    //
    // thread::sleep(ten_millis);
    //
    // let zones = driver.zones();
    // for zone in zones {
    //     println!("\t{:?}", &zone)
    // }

    let data = Data::new(Mutex::new(driver));

    // let players = api.get_players().await?;
    // println!("Got my player: {:?}", &players);
    // for player in &players {
    //     let res = api.get_play_state(player.pid.clone()).await.expect("BUMS!");
    //     println!("{:?}", res);
    //
    //     let (mut r, cmd) = ApiCommand::get_player_volume(player.pid.clone());
    //     api.execute_command(cmd).await;
    //     let res2 = r.await.unwrap();
    //     println!("{:?}", res2);
    // }
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            // .data_factory(||{
            //     rusty_heos::create_api()
            // })
            .service(index)
            .service(echo)
            .wrap(Logger::new("%a %{User-Agent}i"))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;

    Ok(())
}
