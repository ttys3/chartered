use axum::extract;
use bytes::Bytes;
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use chartered_fs::FileSystem;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, convert::TryInto, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Invalid JSON from client: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Invalid body")]
    MetadataParse,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            Self::JsonParse(_) | Self::MetadataParse => StatusCode::BAD_REQUEST,
        }
    }
}

define_error_response!(Error);

#[derive(Serialize, Debug, Default)]
pub struct PublishCrateResponse {
    warnings: PublishCrateResponseWarnings,
}

#[derive(Serialize, Debug, Default)]
pub struct PublishCrateResponseWarnings {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

pub async fn handle(
    extract::Path((_session_key, organisation)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    body: Bytes,
) -> Result<axum::response::Json<PublishCrateResponse>, Error> {
    let (_, (metadata_bytes, crate_bytes)) =
        parse(body.as_ref()).map_err(|_| Error::MetadataParse)?;
    let metadata: Metadata = serde_json::from_slice(metadata_bytes)?;

    let crate_with_permissions = Crate::find_by_name(
        db.clone(),
        user.id,
        organisation.clone(),
        metadata.inner.name.to_string(),
    )
    .await;

    let crate_with_permissions = match crate_with_permissions {
        Ok(v) => Arc::new(v),
        Err(chartered_db::Error::MissingCrate) => {
            let new_crate = Crate::create(
                db.clone(),
                user.id,
                organisation,
                metadata.inner.name.to_string(),
            )
            .await?;
            Arc::new(new_crate)
        }
        Err(e) => return Err(e.into()),
    };

    let file_ref = chartered_fs::Local.write(crate_bytes).await.unwrap();

    crate_with_permissions
        .publish_version(
            db,
            user,
            file_ref,
            hex::encode(Sha256::digest(crate_bytes)),
            metadata_bytes.len().try_into().unwrap(),
            metadata.inner.into_owned(),
            metadata.meta,
        )
        .await?;

    Ok(axum::response::Json(PublishCrateResponse::default()))
}

fn parse(body: &[u8]) -> nom::IResult<&[u8], (&[u8], &[u8])> {
    use nom::{bytes::complete::take, combinator::map_res};
    use std::array::TryFromSliceError;

    let u32_from_le_bytes =
        |b: &[u8]| Ok::<_, TryFromSliceError>(u32::from_le_bytes(b.try_into()?));
    let mut read_u32 = map_res(take(4_usize), u32_from_le_bytes);

    let (rest, metadata_length) = read_u32(body)?;
    let (rest, metadata_bytes) = take(metadata_length)(rest)?;
    let (rest, crate_length) = read_u32(rest)?;
    let (rest, crate_bytes) = take(crate_length)(rest)?;

    Ok((rest, (metadata_bytes, crate_bytes)))
}

#[derive(Deserialize, Debug)]
pub struct Metadata<'a> {
    #[serde(borrow)]
    authors: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    readme_file: Option<Cow<'a, str>>,
    #[serde(borrow)]
    keywords: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    categories: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    license: Option<Cow<'a, str>>,
    #[serde(borrow)]
    license_file: Option<Cow<'a, str>>,
    #[serde(flatten)]
    meta: chartered_types::cargo::CrateVersionMetadata,
    #[serde(flatten)]
    inner: chartered_types::cargo::CrateVersion<'a>,
}
