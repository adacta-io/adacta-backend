use std::io::Cursor;

use anyhow::Error;
use rocket::response::Responder;
use rocket::{
    http::RawStr, http::Status, request::FromParam, response::status::{BadRequest, NotFound}, routes, Request, Response, Route
};

use async_trait::async_trait;
pub use auth::Authorization;

use crate::model::DocId;

mod auth;
mod inbox;
mod repo;
mod search;
mod upload;

pub fn routes() -> Vec<Route> {
    return routes![
        auth::auth,
        repo::bundle,
        repo::fragment,
        search::search,
        inbox::inbox,
        upload::upload_pdf,
    ];
}

impl FromParam<'_> for DocId {
    type Error = Error;

    fn from_param(param: &'_ RawStr) -> Result<Self, Self::Error> { return Ok(param.parse()?); }
}

#[derive(Debug)]
pub(self) struct InternalError(pub Error);

#[async_trait]
impl<'r> Responder<'r> for InternalError {
    async fn respond_to(self, _request: &'r Request<'_>) -> Result<Response<'r>, Status> {
        return Response::build()
            .status(Status::InternalServerError)
            .sized_body(Cursor::new(format!("{:#?}", self.0)))
            .await
            .ok();
    }
}

impl From<Error> for InternalError {
    fn from(err: Error) -> Self { return Self(err); }
}

#[derive(Debug, Responder)]
pub(self) enum ApiError {
    NotFound(NotFound<String>),
    BadRequest(BadRequest<String>),
    InternalError(InternalError),
}

impl ApiError {
    pub const fn not_found(s: String) -> Self { return Self::NotFound(NotFound(s)); }

    pub const fn bad_request(s: String) -> Self { return Self::BadRequest(BadRequest(Some(s))); }
}

impl From<NotFound<String>> for ApiError {
    fn from(r: NotFound<String>) -> Self { return Self::NotFound(r); }
}

impl From<BadRequest<String>> for ApiError {
    fn from(r: BadRequest<String>) -> Self { return Self::BadRequest(r); }
}

impl From<InternalError> for ApiError {
    fn from(r: InternalError) -> Self { return Self::InternalError(r); }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self { return Self::InternalError(err.into()); }
}
