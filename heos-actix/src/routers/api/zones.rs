use actix_web::web::Path;
use actix_web::{guard, web, HttpRequest, HttpResponse, Resource, Scope};
use heos_api::HeosDriver;
use rust_hall::{HalContext, HalResource, Link};
use std::collections::BTreeMap;
use std::fmt::{format, Display};

use crate::domain::zone::Zone;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Error, Value};

const NO_ARGS: Option<&str> = None;

pub struct HeosContext(HttpRequest);

impl Into<HalResource> for Zone {
    fn into(self) -> HalResource {
        HalResource::with_self(format!("/zones{}", self.id())).add_object(self)
    }
}

fn get_zone_links(zone: &Zone, request: &HttpRequest) -> Vec<(String, Link)> {
    let self_link = request.url_for("zone", &[zone.id().to_string()]).unwrap();
    vec![("self".to_string(), self_link.into())]
}

pub async fn list(req: HttpRequest, driver: web::Data<HeosDriver>) -> HttpResponse {
    let zones: Vec<Zone> = Zone::get_zones(&driver);
    let zones = zones
        .into_iter()
        .map(|zone| HalResource::with_self(req.url_for("zone", &[zone.id().to_string()]).unwrap()).add_object(zone));

    let zones_resource = HalResource::with_self(req.url_for_static("zones").unwrap());
    HttpResponse::Ok().json(zones_resource.with_resources("zones", zones))
}

pub async fn details(req: HttpRequest, path: Path<i64>, driver: web::Data<HeosDriver>) -> HttpResponse {
    let player_id = path.into_inner();
    let zones = Zone::get_zones(&driver);
    if let Some(zone) = zones.into_iter().find(|p| p.id() == player_id) {
        HttpResponse::Ok()
            .json(HalResource::with_self(req.url_for("zone", &[zone.id().to_string()]).unwrap()).add_object(zone))
    } else {
        HttpResponse::NotFound().finish()
    }
}
