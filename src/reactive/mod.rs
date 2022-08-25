use std::sync::{Arc, Mutex};

pub type State<T> = Arc<Mutex<T>>;

// from "Hands on functional programming with rust" - Andre Johnson (page 192)
pub struct ReactiveUnit<St, A, B> {
    state: State<St>,
    event_handler: Arc<dyn Fn(&mut St, A) -> B>,
}

impl<St: 'static, A: 'static, B: 'static> ReactiveUnit<St, A, B> {
    pub fn new<F>(state: St, f: F) -> Self
    where
        F: 'static + Fn(&mut St, A) -> B,
    {
        Self {
            state: Arc::new(Mutex::new(state)),
            event_handler: Arc::new(f),
        }
    }

    pub fn bind<G, C>(&self, g: G) -> ReactiveUnit<St, A, C>
    where
        G: Fn(&mut St, B) -> C + 'static,
    {
        let ev = self.event_handler.clone();
        ReactiveUnit {
            state: self.state.clone(),
            event_handler: Arc::new(move |state: &mut St, a: A| {
                let b = ev(state, a);
                let c = g(state, b);
                c
            }),
        }
    }
    // concat two reactive units. Use case!?
    pub fn plus<St2: 'static, C: 'static>(
        &self,
        other: ReactiveUnit<St2, B, C>,
    ) -> ReactiveUnit<(State<St>, State<St2>), A, C> {
        let ev1 = self.event_handler.clone();
        let ev2 = other.event_handler.clone();
        let state1 = self.state.clone();
        let state2 = other.state.clone();
        ReactiveUnit {
            state: Arc::new(Mutex::new((state1, state2))),
            event_handler: Arc::new(move |states: &mut (State<St>, State<St2>), a: A| {
                let mut st1 = states.0.lock().unwrap();
                let r1 = ev1(&mut st1, a);
                let mut st2 = states.1.lock().unwrap();
                let r2 = ev2(&mut st2, r1);
                r2
            }),
        }
    }

    pub fn apply(&self, a: A) -> B {
        let mut current_state = self.state.lock().unwrap();
        (self.event_handler)(&mut current_state, a)
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::ApiCommand;
    use crate::driver::state::DriverState;
    use crate::model::event::HeosEvent;
    use crate::model::Level;
    use crate::model::OnOrOff::Off;
    use crate::model::player::PlayerInfo;
    use crate::model::zone::Player;

    use super::*;

    const PLAYERS_STR: &str = r#"
                    [ {"name": "Heimkino", "pid": 1128532863, "model": "HEOS HomeCinema", "version": "1.583.147", "ip": "192.168.178.34", "network": "wifi", "lineout": 0},
                      {"name": "schöne Box", "pid": -1428708007, "gid": -1899423658, "model": "HEOS 7", "version": "1.583.147", "ip": "192.168.178.35", "network": "wifi", "lineout": 0},
                      {"name": "Küche", "pid": -1899423658, "gid": -1899423658, "model": "HEOS 1", "version": "1.583.147", "ip": "192.168.178.27", "network": "wifi", "lineout": 0}]
                "#;

    fn get_players() -> Vec<PlayerInfo> {
        serde_json::from_str(PLAYERS_STR).unwrap()
    }

    #[test]
    pub fn test_the_bind() {
        fn set_volume(player: &mut Player, volume: Level) -> Player {
            player.volume = Some(volume);
            player.clone()
        }
        let unit1 = ReactiveUnit::new(Player::default(), set_volume);

        let res = unit1.apply(12);
        println!("{:?}", res);
    }

    #[test]
    pub fn simulate_stuff() {
        fn handle_event(driver_state: &mut DriverState, event: HeosEvent) -> Vec<ApiCommand> {
            match event {
                HeosEvent::PlayersChanged => {
                    vec![ApiCommand::GetPlayers, ApiCommand::GetGroups]
                }
                HeosEvent::GroupChanged => {
                    vec![ApiCommand::GetGroups]
                }
                HeosEvent::PlayerVolumeChanged {
                    player_id,
                    level,
                    mute,
                } => {
                    driver_state.update_player(player_id, move |player| {
                        player.volume = Some(level);
                        player.mute = Some(mute);
                    });
                    vec![]
                }
                _ => vec![],
            }
        }
        let mut driver_state = DriverState::default();
        driver_state.set_players(get_players());
        let event_handler = ReactiveUnit::new(driver_state, handle_event);
        let mut commands = vec![];
        commands.extend(event_handler.apply(HeosEvent::PlayersChanged));
        commands.extend(event_handler.apply(HeosEvent::PlayerVolumeChanged {
            player_id: 1128532863,
            mute: Off,
            level: 23,
        }));
        assert_eq!(
            commands,
            vec![ApiCommand::GetPlayers, ApiCommand::GetGroups]
        )
    }
}
