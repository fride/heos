use rusty_heos::api::ApiCommand;
use rusty_heos::HeosResult;

#[tokio::main]
async fn main() -> crate::HeosResult<()> {
    let api = rusty_heos::create_api().await?;
    let players = api.get_players().await?;
    println!("Got my player: {:?}", &players);
    for player in &players {
        let res = api.get_play_state(player.pid.clone()).await.expect("BUMS!");
        println!("{:?}", res);

        let (mut r, cmd) = ApiCommand::get_player_volume(player.pid.clone());
        api.execute_command(cmd).await;
        let res2 = r.await.unwrap();
        println!("{:?}", res2);
    }
    Ok(())
}
