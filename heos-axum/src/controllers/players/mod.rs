use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use heos_api::HeosDriver;
use maud::html;

pub async fn show_players(Extension(driver): Extension<HeosDriver>) -> impl IntoResponse {
    let players = driver.players();
    html!({
        div {
            ol {
                @for player in players {
                    li id=(player.player_id) {
                        p { (player.name) }
                        p { (player.volume)}
                    }
                }
            }
        }
    })
    .into_response()
}

pub fn router(driver: HeosDriver) -> Router {
    Router::new()
        .route("/players", get(show_players))
        .layer(Extension(driver))
}
