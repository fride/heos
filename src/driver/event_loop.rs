use crate::driver::state::DriverState;
use crate::model::event::HeosEvent;
use crate::model::Range;
use crate::util::Shared;
use crate::{Connection, HeosApi};
use tokio_stream::StreamExt;

pub fn start_event_loop(api: HeosApi, connection: Connection, state: Shared<DriverState>) {
    //
    tokio::spawn(async move {
        let events = connection.into_event_stream();
        tokio::pin!(events);
        while let Some(event) = events.next().await {
            match event {
                Ok(event) => {
                    match event {
                        HeosEvent::SourcesChanged => {
                            let sources = api.get_music_sources().await;
                            match sources {
                                Ok(sources) => {
                                    state.with_mutable_state(|s| s.music_sources = sources);
                                }
                                Err(_) => {}
                            }
                        }
                        HeosEvent::PlayersChanged => {
                            let players = api.get_player_infos().await;
                            match players {
                                Ok(players) => {
                                    state.with_mutable_state(|s| s.set_players(players));
                                }
                                Err(_) => {}
                            }
                        }
                        HeosEvent::GroupChanged => {
                            let groups = api.get_groups().await;
                            match groups {
                                Ok(groups) => {
                                    state.with_mutable_state(|state| state.set_groups(groups))
                                }
                                Err(_) => {}
                            }
                        }
                        HeosEvent::PlayerStateChanged {
                            player_id,
                            state: play_state,
                        } => {
                            state.with_mutable_state(|state| {
                                state.set_play_state(player_id, play_state)
                            });
                        }
                        HeosEvent::PlayerNowPlayingChanged { player_id } => {
                            let now_playing = api.get_now_playing_media(player_id).await;
                            match now_playing {
                                Ok(now_playing) => {
                                    state.with_mutable_state(|s| {
                                        s.player_now_playing.insert(player_id, now_playing)
                                    });
                                }
                                Err(_) => {}
                            }
                        }
                        HeosEvent::PlayerNowPlayingProgress { .. } => {}
                        HeosEvent::PlayerPlaybackError { .. } => {}
                        HeosEvent::PlayerVolumeChanged {
                            player_id,
                            level,
                            mute,
                        } => state.with_mutable_state(|s| {
                            s.player_volumes.insert(player_id.clone(), level);
                            s.player_mutes.insert(player_id, mute);
                        }),
                        HeosEvent::PlayerQueueChanged { player_id } => {
                            let queue = api.get_queue(player_id, Range::default()).await;
                            match queue {
                                Ok(queue) => {
                                    state.with_mutable_state(|s| {
                                        s.player_queues.insert(player_id, queue)
                                    });
                                }
                                Err(_) => {}
                            }
                        }
                        HeosEvent::PlayerRepeatModeChanged { .. } => {}
                        HeosEvent::PlayerShuffleModeChanged { .. } => {}
                        HeosEvent::GroupVolumeChanged {
                            group_id,
                            level,
                            mute: _,
                        } => {
                            state.with_mutable_state(|s| {
                                s.group_volumes.insert(group_id, level);
                                //s.grz
                            })
                        }
                        HeosEvent::UserChanged { .. } => {}
                    }
                }
                Err(err) => {
                    println!("BOOOM! {:?}", err);
                }
            }
        }
    });
}
