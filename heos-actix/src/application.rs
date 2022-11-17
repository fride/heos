use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{guard, web, App, HttpServer, Scope};
use heos_api::HeosDriver;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
// TODO NOT The Tokio one!?
use crate::configuration::Settings;
use crate::routers::{api, music_source};
use crate::routers::{
    health_check, home, main_css, zone::details, zone::edit_zone_members_form,
    zone::list as list_zones, zone::new as new_zone,
};

pub struct Application {
    port: u16,
    server: Server,
    driver: HeosDriver,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let heos_address = format!("{}:{}", configuration.heos.host, 1255);
        let listener = TcpListener::bind(&address)?;
        let driver = heos_api::HeosDriver::new(heos_address).await?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, configuration.application.base_url, driver.clone()).await?;
        Ok(Self {
            port,
            server,
            driver,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn run(
    listener: TcpListener,
    base_url: String,
    driver: HeosDriver,
) -> Result<Server, anyhow::Error> {
    let driver = Data::new(driver);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .route("/zones", web::get().to(list_zones))
            .service(api::routes())
            .service(
                web::resource("/zones/{zone_id}")
                    .name("view_zone")
                    .route(web::get().to(details)),
            )
            .route("/music_sources", web::get().to(music_source::list))
            .service(
                web::resource("/zones/{zone_id}/edit_members")
                    .name("edit_members")
                    .guard(guard::Get())
                    .to(edit_zone_members_form),
            )
            .service(
                web::resource("/zones/{zone_id}")
                    .name("new_zone")
                    .guard(guard::Post())
                    .to(new_zone),
            )
            //.route("/zones/{zone_id}/edit_members", web::get().to(edit_zone_members_form))
            .route("/statics/style.scss", web::get().to(main_css))
            .route("/health_check", web::get().to(health_check))
            .app_data(base_url.clone())
            .app_data(driver.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
