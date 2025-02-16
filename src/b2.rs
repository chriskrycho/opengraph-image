use std::fmt;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::header;
use sha1::{Digest, Sha1};

const API_BASE: &str = "https://api.backblazeb2.com";
const API_PATH: &str = "b2api/v3";

pub struct ClientBuilder {
    application_key: String,
    application_key_id: String,
}

impl ClientBuilder {
    pub fn new(application_key_id: String, application_key: String) -> Self {
        Self {
            application_key_id,
            application_key,
        }
    }

    pub async fn authorize(&mut self, client: reqwest::Client) -> Result<Client, Error> {
        let credentials = format!("{}:{}", self.application_key_id, self.application_key);
        let auth_header = format!("Basic {}", BASE64.encode(credentials));

        let resp = client
            .get(format!("{API_BASE}/{API_PATH}/b2_authorize_account"))
            .header(header::AUTHORIZATION, auth_header)
            .send()
            .await
            .map_err(|source| Error::Authorize { source })?
            .error_for_status()?;

        let text = resp
            .text()
            .await
            .map_err(|source| Error::Authorize { source })?;

        let de = &mut serde_json::Deserializer::from_str(&text);
        let auth_response: AuthResponse =
            serde_path_to_error::deserialize(de).map_err(|source| Error::Deserialize { source })?;

        let auth = Auth::from(auth_response);

        Ok(Client { client, auth })
    }
}

pub struct Client {
    auth: Auth,
    client: reqwest::Client,
}

impl Client {
    const FILE_NAME: &str = "X-Bz-File-Name";
    const CONTENT_SHA: &str = "X-Bz-Content-Sha1";

    pub async fn upload_file(&mut self, file_name: &str, data: Vec<u8>) -> Result<(), Error> {
        let file_name = if file_name.starts_with("opengraph/") {
            file_name.to_string()
        } else {
            format!("opengraph/{file_name}")
        };

        let mut latest_err = None;
        for _ in 0..5 {
            let upload = self.get_upload_url().await?;

            let req = self
                .client
                .post(&upload.upload_url)
                .header(header::AUTHORIZATION, upload.authorization_token)
                .header(Client::FILE_NAME, &file_name)
                .header(header::CONTENT_TYPE, "image/png")
                .header(header::CONTENT_LENGTH, data.len())
                .header(Client::CONTENT_SHA, sha1_hash(&data))
                .body(data.clone());

            let response = req
                .send()
                .await
                .map_err(|source| Error::UploadRequest { source })?;

            if response.status().is_success() {
                return Ok(());
            } else {
                let error: ErrorResponse = response.json().await?;
                latest_err = Some(Error::Upload { details: error });
            }
        }

        if let Some(err) = latest_err {
            Err(err)
        } else {
            Ok(())
        }
    }

    async fn get_upload_url(&self) -> Result<UploadUrlResponse, Error> {
        let req_url = format!("{}/{API_PATH}/b2_get_upload_url", self.auth.api_url);
        let response = self
            .client
            .get(req_url)
            .header(header::AUTHORIZATION, &self.auth.token)
            .query(&[("bucketId", &self.auth.bucket_id)])
            .send()
            .await
            .map_err(|source| Error::GetUploadUrl { source })?
            .error_for_status()?;

        let text = response
            .text()
            .await
            .map_err(|source| Error::Authorize { source })?;

        let de = &mut serde_json::Deserializer::from_str(&text);
        let upload_url =
            serde_path_to_error::deserialize(de).map_err(|source| Error::Deserialize { source })?;

        Ok(upload_url)
    }
}

// Note: these are intentionally incomplete! This does *not* ignore unknown
// fields.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthResponse {
    api_info: ApiInfoResponse,
    authorization_token: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiInfoResponse {
    storage_api: StorageApiResponse,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageApiResponse {
    api_url: String,
    bucket_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct Auth {
    api_url: String,
    token: String,
    bucket_id: String,
}

impl From<AuthResponse> for Auth {
    fn from(response: AuthResponse) -> Self {
        Auth {
            api_url: response.api_info.storage_api.api_url,
            token: response.authorization_token,
            bucket_id: response.api_info.storage_api.bucket_id,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadUrlResponse {
    upload_url: String,
    authorization_token: String,
}

fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Request failed: {source}")]
    Request {
        #[from]
        source: reqwest::Error,
    },

    #[error("Upload failed: {source}")]
    UploadRequest { source: reqwest::Error },

    #[error("Deserialize failed: {source}")]
    Deserialize {
        #[from]
        source: serde_path_to_error::Error<serde_json::Error>,
    },

    #[error("Authorization failed: {source}")]
    Authorize { source: reqwest::Error },

    #[error("Upload failed: {details:?}")]
    Upload { details: ErrorResponse },

    #[error("Get upload URL failed: {source}")]
    GetUploadUrl { source: reqwest::Error },
}

#[derive(Debug, serde::Deserialize)]
pub struct ErrorResponse {
    code: String,
    message: String,
    status: u16,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: '{}, ", self.code)?;

        if !self.message.is_empty() {
            write!(f, "message: '{}, ", self.message)?;
        }

        write!(f, "status: {}", self.status)
    }
}
