use axum::{
    extract::{self, FromRequest, RequestParts},
    http::{Request, Response, StatusCode},
};
use chartered_db::ConnectionPool;
use futures::future::BoxFuture;
use std::{
    collections::HashMap,
    task::{Context, Poll},
};
use tower::Service;

#[derive(Clone)]
pub struct AuthMiddleware<S>(pub S);

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.0.clone();
        let mut inner = std::mem::replace(&mut self.0, clone);

        Box::pin(async move {
            let mut req = RequestParts::new(req);

            let params = extract::Path::<HashMap<String, String>>::from_request(&mut req)
                .await
                .unwrap();

            let key = params.get("key").map(String::as_str).unwrap_or_default();

            let db = req
                .extensions()
                .unwrap()
                .get::<ConnectionPool>()
                .unwrap()
                .clone();

            let user = match chartered_db::users::User::find_by_session_key(db, String::from(key))
                .await
                .unwrap()
            {
                Some(user) => std::sync::Arc::new(user),
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(ResBody::default())
                        .unwrap())
                }
            };

            req.extensions_mut().unwrap().insert(user);

            let response: Response<ResBody> = inner.call(req.try_into_request().unwrap()).await?;

            Ok(response)
        })
    }
}
