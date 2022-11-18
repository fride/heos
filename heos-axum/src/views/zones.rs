use crate::models::zones::{NowPlaying, Zone};
use maud::{html, Markup};

pub fn render_zone(zone: &Zone) -> Markup {
    html!({.zones__zone if=(format!("zone{}", zone.id)){
         .zones__zone__header {
            .zones__zone__header__image {
                img src=(zone.now_playing_image()) {}
            }
            .zones__zone__header__name  { (zone.name) }
            .zones__zone__header__song  { (zone.now_playing.song()) }
            .zones__zone__header__artist  { (zone.now_playing.artist()) }
            .zones__zone__header__album  { (zone.now_playing.album()) }
            .zones__zone__header__actions {
                a href="#" {
                    ( "edit" )
                }
                a href="#" {
                    i class=(zone.play_state_class()){}
                }
            }
        }
        .zones__zone__members {
            ol {
                @for (pid, (name, level)) in &zone.members {
                    li {
                        label for=(format!("volume{}", pid)) {
                            (name)
                        }
                        input type="range" name=(format!("member_volume{}", pid))
                              min="0" max="100" value=(level){}
                    }
                }
            }
        }
        .zones__zone__volume {
            input type="range" name=(format!("volume{}", zone.id))
                  min="0" max="100" value=(zone.volume){}
        }
    }})
}

pub fn render_zone_now_playing(zone: &Zone) -> Markup {
    html!({
        div class="zones__zone__now-playing" {
            a href="#" { i class=(zone.play_state_class()) {} }
            ( render_now_playing(&zone.now_playing) )
        }
    })
}

fn render_now_playing(now_playing: &NowPlaying) -> Markup {
    match now_playing {
        NowPlaying::Noting => html!({ (" - ") }),
        NowPlaying::Station {
            image_url,
            station,
            song,
            artist,
            ..
        } => html!({
            img src=(image_url) {}
            p .zones__zone__now-playing__station {
                ( station)
            }
            p .zones__zone__now-playing__song {
                ( song )
            }
            p .zones__zone__now-playing__artist {
                ( artist )
            }
        }),
        NowPlaying::Song {
            image_url,
            song,
            artist,
            album,
            ..
        } => {
            html!({
                img src=(image_url) {}
                p .zones__zone__now-playing__song {
                    ( song )
                }
                p .zones__zone__now-playing__artist {
                    ( artist )
                }
                p .zones__zone__now-playing__album {
                    ( album )
                }
            })
        }
    }
}
