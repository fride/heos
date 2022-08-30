#[macro_use]
extern crate serde_derive;

use std::sync::Mutex;
use std::time::Duration;

use actix_web::http::header::HeaderMap;
use actix_web::{get, post, HttpRequest, HttpResponse, Responder};

use actix_web::web::Data;
use askama::Template;
use pretty_env_logger::env_logger;
use tokio::sync::oneshot;

use rusty_heos::{create_api, HeosApi, HeosResult};
use rusty_heos::driver::*;

mod templates;

#[actix_web::main]
async fn main() -> crate::HeosResult<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let mut connection = rusty_heos::connect(Some("192.168.178.35:1255")).await?;
    // let mut connection = rusty_heos::connect::<&str>(None).await?;

    let driver = Driver::create(connection).await?;
    let players = driver.get_players();

    let x = tokio::spawn(async move {
        loop {
            let players = &driver.get_players();
            for player in players {
                println!("Player: {}", player.to_string());
            }
            for zone in driver.get_zones() {
                println!("Zone: {}", zone.to_string());
            }
            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
    x.await;


    // let api2 = api.clone();
    // let api3 = api.clone();
    // let j1 = tokio::spawn(async move {
    //     let players = api.get_player_infos().await;
    //     println!("Players: {:?}", &players);
    //     players
    // });
    // let j2 = tokio::spawn(async move {
    //     let players = api2.get_player_infos().await;
    //     println!("Players: {:?}", &players);
    //     players
    // });
    // let j3 = tokio::spawn(async move {
    //     let players = api3.get_player_infos().await;
    //     println!("Players: {:?}", &players);
    //     players
    // });
    //
    // j2.await;
    // j1.await;
    // j3.await;

    // let connection = rusty_heos::connect(Some("192.168.178.35:1255")).await?;
    // let mut connection = rusty_heos::connect::<&str>(None).await?;
    // let mut channel : CommandChannel = connection.into();
    //
    // let players = channel.schedule(GetPlayers::new()).await;
    // println!("Players: {:?}", players);
    // //
    // let (s,mut r) = tokio::sync::mpsc::channel::<Command>(12);
    // tokio::spawn(async move {
    //    while let Some(command) = r.recv().await {
    //         command.apply(&mut connection).await;
    //    }
    // });
    // let players = foo(&s).await;
    // println!("Players: {:?}", players);

    //
    // let mut controller = Controller::new(connection).await?;
    //
    // controller.init().await;
    // println!("controller: {:?}", &controller.get_players());
    // println!("controller: {:?}", &controller.get_music_sources());

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
