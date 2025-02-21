mod b2;
mod image;

use std::process;

use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    http::{self, HeaderMap, Response},
    response::IntoResponse,
    routing,
};
use reqwest::header;
use sha1::{Digest, Sha1};
use shuttle_runtime::SecretStore;

const GIT_SHA: &str = env!("GIT_SHA");

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let auth = Auth::try_from(secrets).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    let app_state = AppState { auth };

    let router = Router::new()
        .route("/", routing::get(image))
        .with_state(app_state);

    Ok(router.into())
}

#[derive(Debug, Clone)]
struct AppState {
    auth: Auth,
}

#[axum::debug_handler]
async fn image(
    State(AppState { auth }): State<AppState>,
    Query(qp): Query<QueryParams>,
) -> Result<Response<Body>, Error> {
    let hash = sha1_hash(qp.page_title.as_bytes());
    let file_name = format!("{GIT_SHA}-{hash}.png");

    let mut b2_client = b2::ClientBuilder::new(auth.id, auth.key)
        .authorize(reqwest::Client::new())
        .await?;

    let image_data = match b2_client.download_file(&file_name).await? {
        Some(data) => data,
        None => {
            let data = image::render(&qp.page_title);
            b2_client.upload_file(&file_name, &data).await?;
            data
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=31536000".parse().unwrap(),
    );
    headers.insert(header::ETAG, file_name.parse().unwrap());
    let response = (headers, image_data).into_response();
    Ok(response)
}

fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
#[derive(Clone, Debug)]
struct Auth {
    id: String,
    key: String,
}

impl TryFrom<SecretStore> for Auth {
    type Error = MissingSecrets;

    fn try_from(secrets: SecretStore) -> Result<Self, Self::Error> {
        match (secrets.get("ID"), secrets.get("KEY")) {
            (Some(id), Some(key)) => Ok(Auth { id, key }),
            (Some(_), None) => Err(MissingSecrets::Key),
            (None, Some(_)) => Err(MissingSecrets::Id),
            (None, None) => Err(MissingSecrets::Both),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct QueryParams {
    page_title: String,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    B2 {
        #[from]
        source: b2::Error,
    },
}

#[derive(Debug, thiserror::Error)]
enum MissingSecrets {
    #[error("Missing `KEY`")]
    Key,
    #[error("Missing `ID`")]
    Id,
    #[error("Missing `KEY` and `ID`")]
    Both,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
