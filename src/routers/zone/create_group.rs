use std::collections::BTreeMap;
use actix_web::{HttpResponse, web};
use actix_web::http::header::LOCATION;
use heos_api::{HeosDriver, HeosResult};
use heos_api::types::PlayerId;
use tracing::{error, info};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ZoneEditForm {
    #[serde(default)]
    pub action: String,
    #[serde(flatten)]
    pub member_ids: BTreeMap<String, String>,
}

impl ZoneEditForm {
    pub fn member_ids(&self) -> Vec<PlayerId> {
        let mut pids = vec![];
        for (pid, on_or_off) in &self.member_ids {
            info!("Pid: `{}` :: `{}`", &pid, &on_or_off);
            if on_or_off == "on" {
                if let Ok(pid) = pid.parse::<i64>() {
                    pids.push(pid);
                }
            }
        }
        pids
    }
}

pub async fn new(path: web::Path<i64>,
                 params: web::Form<ZoneEditForm>,
                 driver: web::Data<HeosDriver>) -> HttpResponse {
    let members_ids = params.member_ids();
    let leader = path.into_inner();

    match driver.create_group(leader, members_ids).await {
        Ok(_) => {
            HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish()
        }
        Err(err) => {
            error!("Grouping players failed! {:?}", &err);
            HttpResponse::InternalServerError()
                .body("Grouping players failed")
        }
    }
}
