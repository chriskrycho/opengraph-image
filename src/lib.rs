mod b2;
mod image;

use std::{env, string::FromUtf8Error};

use sha1::{Digest, Sha1};

use worker::{Context, Cors, Env, Headers, HttpRequest, Method, Response, event};

const GIT_SHA: &str = env!("GIT_SHA");

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> Result<Response, Error> {
    console_error_panic_hook::set_once();

    if req.method() == http::Method::OPTIONS {
        return cors(&env);
    }

    let auth = get_auth(&env)?;

    let uri = req.uri();
    let from_path = uri
        .path()
        .strip_prefix('/')
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .map(|s| s.to_string());

    let from_query = uri
        .query()
        .and_then(|qp| serde_urlencoded::from_str::<QueryParams>(qp).ok())
        .map(|qp| qp.page_title);

    let page_title = match (from_path, from_query) {
        (Some(title), None) | (None, Some(title)) => Ok(title),
        (Some(_), Some(_)) => Err(Error::BothPathAndQuery),
        (None, None) => Err(Error::MissingPageTitle),
    }?;

    let page_title = urlencoding::decode(&page_title)?;
    get_image(auth, &page_title).await
}

#[derive(Debug, serde::Deserialize)]
struct QueryParams {
    page_title: String,
}

fn cors(env: &Env) -> Result<Response, Error> {
    let is_prod = env.secret("DEV").map(|s| s.to_string()).unwrap_or_default() == "true";
    let origins = if is_prod {
        ["https://*.chriskrycho.com"]
    } else {
        ["http://localhost:*"]
    };

    let cors = Cors::new()
        .with_methods([Method::Get])
        .with_origins(origins);

    Response::empty()
        .and_then(|res| res.with_cors(&cors))
        .map_err(|source| Error::Worker { source })
}

fn get_auth(env: &Env) -> Result<Auth, Error> {
    let id = env
        .secret("B2_ID")
        .map_err(|source| Error::Secrets { source })?
        .to_string();
    let key = env
        .secret("B2_KEY")
        .map_err(|source| Error::Secrets { source })?
        .to_string();
    Ok(Auth { id, key })
}

async fn get_image(auth: Auth, page_title: &str) -> Result<Response, Error> {
    let hash = sha1_hash(page_title.as_bytes());
    let file_name = format!("{GIT_SHA}-{hash}.png");

    let mut b2_client = b2::ClientBuilder::new(auth.id, auth.key)
        .authorize(reqwest::Client::new())
        .await?;

    let image_data = match b2_client.download_file(&file_name).await? {
        Some(data) => data,
        None => {
            let data = image::render(page_title);
            b2_client.upload_file(&file_name, &data).await?;
            data
        }
    };

    let mut headers = Headers::new();
    headers.set("Content-Type", "image/png")?;
    headers.set("Cache-Control", "public, max-age=31536000")?;
    headers.set("ETag", &file_name)?;
    Ok(worker::Response::from_bytes(image_data)?.with_headers(headers))
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

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    B2 {
        #[from]
        source: b2::Error,
    },

    #[error("Invalid or missing secrets: {source}")]
    Secrets { source: worker::Error },

    #[error(transparent)]
    Worker {
        #[from]
        source: worker::Error,
    },

    #[error("Could not deserialize query params: {source}")]
    InvalidPath {
        #[from]
        source: FromUtf8Error,
    },

    #[error("Missing page title")]
    MissingPageTitle,

    #[error("Requested both ?page_title and /:page_title")]
    BothPathAndQuery,
}
