use axum::{extract, Json};
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    uuid::Uuid,
    ConnectionPool,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use crate::endpoints::ErrorResponse;

#[derive(Serialize)]
pub struct GetResponse {
    allowed_permissions: &'static [&'static str],
    members: Vec<GetResponseMember>,
}

#[derive(Deserialize, Serialize)]
pub struct GetResponseMember {
    uuid: Uuid,
    username: String,
    permissions: Permission,
}

pub async fn handle_get(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<Json<GetResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let members = crate_with_permissions
        .members(db)
        .await?
        .into_iter()
        .map(|(user, permissions)| GetResponseMember {
            uuid: user.uuid.0,
            username: user.username,
            permissions,
        })
        .collect();

    Ok(Json(GetResponse {
        allowed_permissions: Permission::names(),
        members,
    }))
}

#[derive(Deserialize)]
pub struct PutOrPatchRequest {
    user_uuid: chartered_db::uuid::Uuid,
    permissions: Permission,
}

pub async fn handle_patch(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutOrPatchRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    let affected_rows = crate_with_permissions
        .update_permissions(db, action_user.id, req.permissions)
        .await?;
    if affected_rows == 0 {
        return Err(Error::UpdateConflictRemoved);
    }

    Ok(Json(ErrorResponse { error: None }))
}

pub async fn handle_put(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<PutOrPatchRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    crate_with_permissions
        .insert_permissions(db, action_user.id, req.permissions)
        .await?;

    Ok(Json(ErrorResponse { error: None }))
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    user_uuid: chartered_db::uuid::Uuid,
}

pub async fn handle_delete(
    extract::Path((_session_key, organisation, name)): extract::Path<(String, String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Json(req): extract::Json<DeleteRequest>,
) -> Result<Json<ErrorResponse>, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    let action_user = User::find_by_uuid(db.clone(), req.user_uuid)
        .await?
        .ok_or(Error::InvalidUserId)?;

    crate_with_permissions
        .delete_member(db, action_user.id)
        .await?;

    Ok(Json(ErrorResponse { error: None }))
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Permissions update conflict, user was removed as a member of the crate")]
    UpdateConflictRemoved,
    #[error("An invalid user id was given")]
    InvalidUserId,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            Self::UpdateConflictRemoved => StatusCode::CONFLICT,
            Self::InvalidUserId => StatusCode::BAD_REQUEST,
        }
    }
}

define_error_response!(Error);
