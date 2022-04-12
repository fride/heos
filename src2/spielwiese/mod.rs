use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use druid::{AppDelegate, Command, Data, DelegateCtx, Env, Handled, Lens, Selector, Target};
use im;
use crate::model::{Level, Repeat, Shuffle};
use crate::player::{NowPlayingMedia, PlayState};
use crate::{player, PlayerUpdate};

type Ref<A> = Rc<RefCell<A>>;

const SET_PLAYERS: Selector<PlayerUpdate> = Selector::new("player-update");

mod state;
mod widgets;

mod api_experiment {
    use crate::model::group::GroupInfo;
    use crate::model::PlayerId;
    use crate::player::{PlayerInfo, PlayerNowPlayingMedia, PlayerPlayMode, PlayerPlayState, PlayerVolume};

    enum State {
        Locating,
        Initializing,
        Loaded{
            players: Vec<PlayerInfo>,
            groups: Vec<GroupInfo>,
            player_volumes: Vec<PlayerVolume>, // this must be a map
            player_now_playing: Vec<PlayerNowPlayingMedia> // this must be a map!
        }
    }
    enum StateChange{
        PlayersChanged(Vec<PlayerInfo>),
        GroupsChanged(Vec<GroupInfo>),
        PlayerVolumeChanged(PlayerVolume),
        PlayerModeChanged(PlayerPlayMode),
        // this is a snapshot of the state.
        Loaded{
            players: Vec<PlayerInfo>,
            groups: Vec<GroupInfo>,
            player_volumes: Vec<PlayerVolume>, // this must be a map
            player_now_playing: Vec<PlayerNowPlayingMedia> // this must be a map!
        }
    }

    pub struct HeosDriverChannel (tokio::sync::mpsc::Sender<HeosDriverMessage>, tokio::sync::mpsc::Receiver<State>);

    enum HeosDriverMessage{
        GetState,
        SetPlayState(PlayerPlayState),
        SetPlayMode(PlayerPlayMode),
        WatchState(tokio::sync::watch::Sender<StateChange>)
    }

    // as we are always talking to something that breaks use async!
    trait HeosDriver {
        async fn state(&mut self) -> State;
        async fn set_play_state(&mut self, new_state: PlayerPlayState);
        async fn set_play_mode(&mut self, new_mode: PlayerPlayMode);
    }
}
