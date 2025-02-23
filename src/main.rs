mod b2;
mod image;

use std::{env, io, path::PathBuf};

use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    http::{self, HeaderMap, Response},
    response::IntoResponse,
    routing,
};
use http::{HeaderValue, Method};
use reqwest::header;
use sha1::{Digest, Sha1};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

const GIT_SHA: &str = env!("GIT_SHA");
const SECRETS: &str = "Secrets.toml";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let secrets_file = tokio::fs::read_to_string(SECRETS)
        .await
        .map_err(|source| Error::Io {
            path: SECRETS.into(),
            source,
        })?;

    let auth: Auth = toml::from_str(&secrets_file)?;

    let is_prod = env::var("ON_RENDER").unwrap_or_default() == "true";


    let state = AppState { auth };
    let allowed = if is_prod {
        "https://*.chriskrycho.com"
    } else {
        "http://localhost:*"
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(HeaderValue::from_str(allowed).unwrap());

    let app = Router::new()
        .route("/", routing::get(image))
        .with_state(state)
        .layer(cors);

    let port = env::var("PORT").unwrap_or("10000".to_string());
    let host = if is_prod { "0.0.0.0" } else { "127.0.0.1" };
    let listener = TcpListener::bind(format!("{host}:{port}"))
        .await
        .map_err(|source| Error::Port { port, source })?;

    axum::serve(listener, app)
        .await
        .map_err(|source| Error::Serve { source })
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

#[derive(Clone, Debug, serde::Deserialize)]
struct Auth {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "KEY")]
    key: String,
}

#[derive(Debug, serde::Deserialize)]
struct QueryParams {
    page_title: String,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("IO error at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },

    #[error("Could not bind to port {port}")]
    Port { port: String, source: io::Error },

    #[error(transparent)]
    B2 {
        #[from]
        source: b2::Error,
    },

    #[error("Invalid or missing secrets: {source}")]
    Secrets {
        #[from]
        source: toml::de::Error,
    },

    #[error("Could not serve app: {source}")]
    Serve { source: io::Error },
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
