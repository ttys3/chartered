#[derive(serde::Serialize)]
pub struct ErrorResponse {
    error: Option<String>,
}

macro_rules! define_error_response {
    ($error:ty) => {
        impl crate::middleware::logging::GenericError for $error {}

        impl axum::response::IntoResponse for $error {
            type Body = axum::body::Full<axum::body::Bytes>;
            type BodyError = <Self::Body as axum::body::HttpBody>::Error;

            fn into_response(self) -> axum::http::Response<Self::Body> {
                let body = serde_json::to_vec(&crate::endpoints::ErrorResponse {
                    error: Some(self.to_string()),
                })
                .unwrap();

                let mut res = axum::http::Response::new(axum::body::Full::from(body));
                *res.status_mut() = self.status_code();
                res.headers_mut().insert(
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::HeaderValue::from_static("application/json"),
                );
                res.extensions_mut()
                    .insert::<Box<dyn crate::middleware::logging::GenericError>>(Box::new(self));
                res
            }
        }
    };
}

pub mod cargo_api;
pub mod web_api;
