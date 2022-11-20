use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub fn render<F>(f: F) -> Html<&'static str>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), std::io::Error>,
{
    let mut buf = Vec::new();
    f(&mut buf).expect("Error rendering template");
    let html: String = String::from_utf8_lossy(&buf).into();

    Html(Box::leak(html.into_boxed_str()))
}
pub struct RenderTemplate<T: FnOnce(&mut Vec<u8>) -> std::io::Result<()>>(pub T);

impl<T: FnOnce(&mut Vec<u8>) -> std::io::Result<()>> IntoResponse for RenderTemplate<T> {
    fn into_response(self) -> axum::response::Response {
        let mut buf = Vec::new();
        match self.0(&mut buf) {
            Ok(()) => Html(buf).into_response(),
            Err(_e) => {
                // TODO: logging
                (StatusCode::INTERNAL_SERVER_ERROR, "Render failed").into_response()
            },
        }
    }
}
