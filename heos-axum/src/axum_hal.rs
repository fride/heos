use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::BufMut;
use rust_hall::HalResource;

pub struct HalJson(HalResource);
impl HalJson {
    pub fn new(resource: HalResource) -> Self {
        HalJson(resource)
    }
}
impl IntoResponse for HalJson {
    fn into_response(self) -> Response {
        let mut buf = bytes::BytesMut::new().writer();
        match serde_json::to_writer(&mut buf, &self.0.to_json()) {
            Ok(()) => (
                [(header::CONTENT_TYPE, HeaderValue::from_static("application/hal+json"))],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}
