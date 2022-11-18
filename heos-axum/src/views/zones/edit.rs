use axum::response::{IntoResponse, Response};

use crate::views::pages::page;
use heos_api::types::player::HeosPlayer;
use heos_api::types::PlayerId;
use maud::{html, Markup};
use tracing::log::info;

#[derive(Debug)]
pub struct Member {
    pub id: PlayerId,
    pub name: String,
    pub checked: bool,
}

#[derive(Debug)]
pub struct EditZoneMembers {
    pub zone_name: String,
    pub zone_id: PlayerId,
    pub members: Vec<Member>,
}

impl EditZoneMembers {
    pub fn new<C>(player: HeosPlayer, members: C) -> Self
    where
        C: IntoIterator<Item = HeosPlayer>,
    {
        let mut me = EditZoneMembers {
            zone_name: player.name,
            zone_id: player.player_id,
            members: vec![],
        };
        for player in members {
            me.add_member(player)
        }
        me
    }
    pub fn add_member(&mut self, member: HeosPlayer) {
        info!("Adding member {:?}", &member.name);
        if member.player_id == self.zone_id {
            info!("Not adding myself");
            return;
        }
        self.members.push(Member {
            id: member.player_id,
            name: member.name,
            checked: member.in_group.filter(|p| *p == self.zone_id).is_some(),
        });
    }

    pub fn render_html(&self) -> Markup {
        let action = format!("/zones/{}", self.zone_id);
        page(html!({
            .zone-edit-members {
                form method="post" action=(action)
                    hx-post=(action) hx-target="#zones"
                {
                    h3 { (self.zone_name) }
                    @for member in &self.members {
                        div class="zone-edit-members__member inputGroup" {
                            @let name=format!("{}", member.id);
                            @if member.checked {
                                input type="checkbox" name=(name) id=(name) checked;
                            }@else {
                                input type="checkbox" name=(name) id=(name);
                            }
                            label for=(name) { (member.name) }
                        }

                    }
                    .zone-edit-members__footer {
                        button type="submit" value="go" .button {
                            ("go")
                        }
                        a href="/zones" .button {
                            ("cancel")
                        }
                    }

                }
            }
        }))
    }
}

impl IntoResponse for EditZoneMembers {
    fn into_response(self) -> Response {
        self.render_html().into_response()
    }
}
