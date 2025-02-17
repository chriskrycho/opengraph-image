mod b2;
mod image;

use std::io;

const GIT_SHA: &str = env!("GIT_SHA");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let data = image::render("Tidy First? A Personal Exercise in Empirical Software Design");
    let file_name = "testing.png";

    let auth_file = std::fs::read_to_string("Secrets.toml").map_err(|source| Error::Io {
        message: String::from("Could not read Secrets.toml"),
        source,
    })?;
    let auth: Auth = toml::from_str(&auth_file)?;

    b2::ClientBuilder::new(auth.id, auth.key)
        .authorize(reqwest::Client::new())
        .await?
        .upload_file(file_name, data)
        .await?;

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]

fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
struct Auth {
    id: String,
    key: String,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    B2 {
        #[from]
        source: b2::Error,
    },

    #[error("{message}: {source}")]
    Io { message: String, source: io::Error },

    #[error("Could not deserialize secrets")]
    DeserializeSecrets {
        #[from]
        source: toml::de::Error,
    },
}
