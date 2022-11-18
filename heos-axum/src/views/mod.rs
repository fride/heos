use maud::Markup;

pub mod pages;

pub mod browse;
pub mod sources;
pub mod media;
pub mod zones;

pub trait RenderHtml {
    fn render_html(&self) -> Markup;
}
