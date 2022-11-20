use maud::Markup;

pub mod pages;

pub mod browse;
pub mod media;
pub mod sources;
pub mod zones;

pub trait RenderHtml {
    fn render_html(&self) -> Markup;
}
