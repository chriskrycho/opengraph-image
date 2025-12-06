mod b2;
mod image;

use std::{env, string::FromUtf8Error};

use sha1::{Digest, Sha1};
use worker::{Cache, Context, Cors, Env, Headers, HttpRequest, Method, Response, event};

pub use image::{Content, render};

const GIT_SHA: &str = env!("GIT_SHA");

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, ctx: Context) -> Result<Response, Error> {
    console_error_panic_hook::set_once();

    if req.method() == http::Method::OPTIONS {
        return cors(&env);
    }

    let uri = req.uri();

    // Explicitly use Cloudflare's cached value if I can. Incorporate the Git
    // SHA for this build as a way of busting Cloudflare's cache for the same
    // set of page title and query when I have updated this tool. (Otherwise, I
    // will always serve the same images no matter how I change this!)
    let cache = Cache::default();
    let cache_key = uri.to_string() + GIT_SHA;
    if let Some(resp) = cache.get(&cache_key, false).await? {
        return Ok(resp);
    }

    let title_from_path = uri
        .path()
        .strip_prefix("/page-title/")
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .map(|s| s.to_string());

    let qps = uri
        .query()
        .and_then(|qp| serde_urlencoded::from_str::<QueryParams>(qp).ok());

    let page_title = match (&title_from_path, qps.as_ref().map(|qps| &qps.page_title)) {
        (Some(title), None) | (None, Some(title)) => {
            urlencoding::decode(title).map_err(Error::from)
        }
        (Some(_), Some(_)) => Err(Error::BothPathAndQuery),
        (None, None) => Err(Error::MissingPageTitle),
    }?
    .to_string();

    let subtitle = qps
        .as_ref()
        .and_then(|qps| qps.subtitle.as_ref())
        .map(|subtitle| urlencoding::decode(subtitle))
        .transpose()?
        .map(|s| s.to_string());

    let title = urlencoding::decode(&page_title)?.to_string();

    let auth = get_auth(&env)?;
    let mut response = get_image(auth, Content { title, subtitle }).await?;

    // Let the caching work happen while returning the response. (This is the
    // canonical example for the `wait_util` API, in fact.)
    let for_cache = response.cloned()?;
    ctx.wait_until(async move {
        cache
            .put(&cache_key, for_cache)
            .await
            .unwrap_or_else(|cause| panic!("Failed to cache '{page_title}': {cause}"));
    });

    Ok(response)
}

#[derive(Debug, serde::Deserialize)]
struct QueryParams {
    page_title: String,
    subtitle: Option<String>,
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

async fn get_image<'a>(auth: Auth, content: Content) -> Result<Response, Error> {
    let mut text_as_bytes = content.title.as_bytes().to_owned();
    if let Some(subtitle) = &content.subtitle {
        text_as_bytes.extend_from_slice(subtitle.as_bytes());
    }

    let hash = sha1_hash(&text_as_bytes);
    let file_name = format!("{GIT_SHA}-{hash}.png");

    let mut b2_client = b2::ClientBuilder::new(auth.id, auth.key)
        .authorize(reqwest::Client::new())
        .await?;

    let image_data = match b2_client.download_file(&file_name).await? {
        Some(data) => data,
        None => {
            let data = image::render(content);
            b2_client.upload_file(&file_name, &data).await?;
            data
        }
    };

    let headers = Headers::new();
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
