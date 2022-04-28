#[macro_use]
extern crate serde_derive;

use actix_files as fs;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use std::sync::Mutex;
use actix_web::guard::{Guard, GuardContext};
use actix_web::http::header;
use actix_web::http::header::HeaderValue;

use pretty_env_logger::env_logger;

use askama::Template;
use rusty_heos::{HeosDriver, HeosResult};
use rusty_heos::command::different_structs::HeosCommandExecutor;
use rusty_heos::command::different_structs::HeosCommands::{GetPlayers, GetPlayerVolume};
use rusty_heos::model::player::PlayerInfo;
use rusty_heos::model::zone::Player;

mod templates;


//
// #[derive(Command)]
// #[result="Vec<PlayerInfo>"]
// #[command="player/get_players"]
// pub struct GetPlayersFoo;

// struct HttpGuard;
// impl Guard for HttpGuard {
//     fn check(&self, ctx: &GuardContext<'_>) -> bool {
//         ctx.head()
//             .headers()
//             .get("Accept")
//             .map_or(false, |hv| hv.as_bytes().contains("text/html"))
//     }
// }
//
// struct JsonGuard;

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
    let template = templates::ZonesTemplate::new(zones);
    println!("Zones!!!");
    let s = template.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/players")]
async fn all_players( req: HttpRequest,data: Data<Mutex<HeosDriver>>) -> actix_web::Result<HttpResponse> {
    println!("Got /players");
    let data = data.lock().unwrap();
    println!("Got /players 1");
    let players = data.players();
    println!("Got /players 2");
    let headers = req.headers();
    println!("Got /players 3");
    if headers.get("Accept")
        //TODO this is ugly!
        .map_or(false, |header| header.to_str().unwrap().contains("application/json")) {
        let json = serde_json::to_string_pretty(&players).unwrap();
        Ok(HttpResponse::Ok().content_type("application/json").body(json))
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

    {
        use tokio::sync::mpsc;
        use rusty_heos::model::group::{GroupInfo, GroupVolume};
        use rusty_heos::command::different_structs::*;
        let mut connection =  rusty_heos::connect(Some("192.168.178.35:1255")).await?;
        let (commands, mut results, mut errors) = create_command_handler(connection);
        let h1 = tokio::spawn(async move {
            while let Some(response) = results.recv().await {
                println!("reeponse: {:?}", response);
            }
        });
        let h2 = tokio::spawn(async move {
            while let Some(response) = errors.recv().await {
                println!("reeponse: {:?}", response);
            }
        });
        let _ = commands.send(HeosCommands::GetPlayers(GetPlayers)).await;
        let _ = commands.send(HeosCommands::GetPlayerVolume(GetPlayerVolume{pid:-12})).await;
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    // let connection = rusty_heos::connect(Some("192.168.178.35:1255")).await?;
    let mut connection = rusty_heos::connect::<&str>(None ).await?;


    {
        use rusty_heos::command::{ExecutableHeosCommand, HeosCommand};
        let command = rusty_heos::command::get_players_command();
        let result : Vec<PlayerInfo> = command.parse_payload(&mut connection).await?;
        println!("Got Result: {:?}", result);
    }
    {
        use rusty_heos::command::different_structs::*;
        let command = GetPlayers;
        let result = connection.execute(command).await?;
        println!("Got Result: {:?}", result);
    }

    let driver = rusty_heos::create_driver(connection).await?;
    driver.init().await;

    println!("Starting web service");
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
            .service(all_players)
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
