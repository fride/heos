use crate::driver::{Player};
use maud::{html, Markup};

pub trait ToHtml {
    fn to_html(&self) -> Markup;
}

impl ToHtml for Player {
    fn to_html(&self) -> Markup {
        self.visit(|p| {
            html! {
                h3 { (p.info.name) }
                p {

                }
            }
        })
    }
}
