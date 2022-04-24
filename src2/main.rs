#[macro_use]
extern crate serde_derive;
// extern crate futures;
extern crate serde_json;
extern crate serde_qs as qs;

use std::collections::BTreeMap;

use druid::{Command, Selector, Target};

pub use error::{HeosError, HeosErrorCode};

use crate::api::PlayerUpdate;
use crate::model::event::HeosEvent;
use crate::model::player;

mod error;
pub mod model;

mod spielwiese;

pub type HeosResult<T> = Result<T, HeosError>;

pub mod api;
mod heos_client;
mod client;

const SET_PLAYERS: Selector<BTreeMap<i64, player::PlayerInfo>> = Selector::new("set_players");


pub mod background {
    use druid::{Command, Selector, Target};

    use super::*;

    pub fn run(rt: &tokio::runtime::Runtime, event_sink: druid::ExtEventSink) {
        rt.spawn(async move {
            println!("in the tokio runtime!");
            ask_stuff(event_sink).await;
            println!("exiting the tokio runtime!");
        });
        println!("Ëxiting ..\n\n\n\n");
    }

    async fn ask_stuff(event_sink: druid::ExtEventSink) {
        println!("in the tokio one deeper!");
        let (api,
            results,
            errors) = api::find().await.expect("Bum");
        println!("connected to api");
        let client = heos_client::HeosClient::new(
            api,
            results,
            errors,
        );
        println!("Init HEOS Driver");
        client.init().await;
        loop {
            println!("Updating state");
            use std::time::Duration;
            use tokio::time::sleep;
            sleep(Duration::from_millis(1000)).await;
            let players = client.get_players().await;

            println!("{:?}", &players);
            event_sink.submit_command(
                SET_PLAYERS,
                players,
                Target::Auto).expect("BOING");
        }
    }
}


mod ui {
    use std::collections::BTreeMap;

    use druid::{AppDelegate, AppLauncher, ExtEventSink, BoxConstraints, Command, Data, DelegateCtx, Env, Event, EventCtx, Handled, LayoutCtx, LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, PlatformError, Size, Target, UpdateCtx, Widget, WidgetExt, WindowDesc};
    use druid::widget::{Button, Flex, Label};
    use crate::model::group::{GroupInfo, GroupVolume};
    use crate::model::Level;
    use crate::model::player::{NowPlayingMedia, PlayerInfo};
    use crate::{player, SET_PLAYERS};

    // #[derive(Clone, Data, Default)]
    // pub struct AppState {
    //     players: BTreeMap<i64, PlayerInfo>,
    //     groups: BTreeMap<i64, GroupInfo>,
    //     player_volumes: BTreeMap<i64, Level>,
    //     group_volumes: BTreeMap<i64, Level>,
    //     last_error: Option<String>,
    //     last_command: Option<String>,
    //     now_playing: BTreeMap<i64, NowPlayingMedia>,
    // }

    #[derive(Clone, Data, Default)]
    pub struct Counter(i32);

    struct MySillyDelegate;

    impl AppDelegate<Counter> for MySillyDelegate {
        fn command(&mut self, ctx:
        &mut DelegateCtx, target: Target, cmd: &Command, data: &mut Counter, env: &Env) -> Handled {
            if let Some(value) =  cmd.get(SET_PLAYERS) {
                data.0 = value.len() as i32;
                println!("Got an event in the UI!");
                Handled::Yes
            } else {
                Handled::No
            }
        }
    }

    pub fn setup() -> Result<(AppLauncher<Counter>, ExtEventSink), PlatformError> {
        // Window builder. We set title and size
        let main_window = WindowDesc::new(ui_builder)
            .title("Hello, Druid!")
            .window_size((200.0, 100.0));


        let launcher = AppLauncher::with_window(main_window);
            //.use_simple_logger(); // Neat!
        let event_sink = launcher.get_external_handle();
        print!("Starting ap\n\n\n");
        Ok((launcher
            .delegate(MySillyDelegate), event_sink))
    }

    fn ui_builder() -> impl Widget<Counter> {
        // The label text will be computed dynamically based on the current locale and count
        let text = LocalizedString::new("hello-counter")
            .with_arg("count", |data: &Counter, _env| (*data).0.into());
        let label = Label::new(text).padding(5.0).center();

        // Two buttons with on_click callback
        let button_plus = Button::new("+1")
            .on_click(|_ctx, data: &mut Counter, _env| (*data).0 += 1)
            .padding(5.0);
        let button_minus = Button::new("-1")
            .on_click(|_ctx, data: &mut Counter, _env| (*data).0 -= 1)
            .padding(5.0);

        // Container for the two buttons
        let flex = Flex::row()
            .with_child(button_plus)
            .with_spacer(1.0)
            .with_child(button_minus);

        // Container for the whole UI
        Flex::column()
            .with_child(label)
            .with_child(flex)
    }
}

use std::time::Duration;
fn main() -> crate::HeosResult<()> {

    {
        use std::sync::Arc;
        // playing with arc!
        let mut foo = Arc::new(vec![1,2,3]);
        let mut foo2 = foo.clone();
        print!("a == b? {}", !Arc::ptr_eq(&foo, &foo2));
        let foo3 = Arc::get_mut(&mut foo).unwrap();
        foo3.push(3);
        print!("a == b? {}", !Arc::ptr_eq(&foo, &foo2));
    }
    // let (api, mut results, mut errors) = api::connect("192.168.178.27:1255").await?;
    // TODO naming is totally silly!
    let rt = tokio::runtime::Runtime::new().unwrap();
    // Data to be used in the app (=state)
    let data = ui::Counter::default();
    let (launcher, event_sink) = ui::setup().expect("Kawummmms!");
    background::run(&rt, event_sink);
    launcher.launch(data).unwrap();
    rt.shutdown_timeout(Duration::from_millis(100));
    Ok(())

}
