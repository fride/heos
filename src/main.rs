#[macro_use]
extern crate serde_derive;

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use pretty_env_logger::env_logger;

use rusty_heos::api::{ApiCommand, HeosApi};
use rusty_heos::HeosResult;

mod ui;

#[get("/")]
async fn index(data: Data<Mutex<HeosApi>>) -> String {
    let data = data.lock().unwrap();
    let players = data.get_players().await.unwrap();
    let groups = data.get_groups().await.unwrap();
    let json: ui::Players = (players, groups).into();
    serde_json::to_string_pretty(&json).unwrap()
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
    let api = rusty_heos::create_api().await?;
    let _ = api.init().await?;
    println!("Why no events!\n\n\n\n\n");
    let data = Data::new(Mutex::new(api));

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
