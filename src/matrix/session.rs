use std::path::{Path, PathBuf};

use color_eyre::Result;
use matrix_sdk::{Client, authentication::matrix::MatrixSession, ruma::exports::serde_json};
use rand::{Rng, SeedableRng, distributions::Alphanumeric, rngs::StdRng};
use serde::{Deserialize, Serialize};
use tokio::fs;
use url::Url;

/// The data needed to re-build a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSession {
    pub(crate) homeserver: Url,
    /// The path of the database.
    pub(crate) db_path: PathBuf,
    /// The passphrase of the database.
    pub(crate) passphrase: String,
}

impl ClientSession {
    pub const fn new(homeserver: Url, db_path: PathBuf, passphrase: String) -> Self {
        Self {
            homeserver,
            db_path,
            passphrase,
        }
    }
}

/// The full session to persist.
#[derive(Debug, Serialize, Deserialize)]
pub struct FullSession {
    /// The data to re-build the client.
    pub(crate) client_session: ClientSession,

    /// The Matrix user session.
    pub(crate) user_session: MatrixSession,

    /// The latest sync token.
    ///
    /// It is only needed to persist it when using `Client::sync_once()` and we
    /// want to make our syncs faster by not receiving all the initial sync
    /// again.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sync_token: Option<String>,
}

impl FullSession {
    pub const fn new(
        client_session: ClientSession,
        user_session: MatrixSession,
        sync_token: Option<String>,
    ) -> Self {
        Self {
            client_session,
            user_session,
            sync_token,
        }
    }
}

pub async fn build_client(data_dir: &Path, homeserver: Url) -> Result<(Client, ClientSession)> {
    let mut rng = StdRng::from_entropy();

    // Generating a subfolder for the database is not mandatory, but it is useful if
    // you allow several clients to run at the same time. Each one must have a
    // separate database, which is a different folder with the SQLite store.
    let db_subfolder: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let db_path = data_dir.join(db_subfolder);

    // Generate a random passphrase.
    let passphrase: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let client = Client::builder()
            .homeserver_url(&homeserver)
            // We use the SQLite store, which is enabled by default. This is the crucial part to
            // persist the encryption setup.
            // Note that other store backends are available and you can even implement your own.
            .sqlite_store(&db_path, Some(&passphrase))
            .build()
            .await?;

    let client_session = ClientSession::new(homeserver, db_path, passphrase);

    Ok((client, client_session))
}

/// Persist the sync token for a future session.
/// Note that this is needed only when using `sync_once`. Other sync methods get
/// the sync token from the store.
pub async fn persist_sync_token(session_file: &Path, sync_token: String) -> Result<()> {
    let serialized_session = fs::read_to_string(session_file).await?;
    let mut full_session: FullSession = serde_json::from_str(&serialized_session)?;

    full_session.sync_token = Some(sync_token);
    let serialized_session = serde_json::to_string(&full_session)?;
    fs::write(session_file, serialized_session).await?;

    Ok(())
}
