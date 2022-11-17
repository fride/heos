use actix_web::{guard, Scope, web};

mod zones;
mod browse;

pub fn routes() -> Scope {
    web::scope("/api").service(
        web::scope("/zones")
            .service(web::resource("").name("zones").guard(guard::Get()).to(zones::list))
            .service(
                web::resource("/{zone_id}")
                    .name("zone")
                    .guard(guard::Get())
                    .to(zones::details),
            ),
    ).service(
        web::scope("/browse")
            .service( web::resource("")
                    .name("music_sources")
                    .guard(guard::Get())
                    .to(browse::list))
            .service(web::resource("/{sid}")
                .name("browse")
                .to(browse::details))
            .service(web::resource("/{sid}/{cid}")
                .name("browse_container")
                .guard(guard::Get())
                .to(browse::container))
    )
}
