use std::fmt;

use color_eyre::{
    Result,
    eyre::{ContextCompat, bail},
};
use matrix_sdk::{Client, ruma::api::client::session::get_login_types::v3::IdentityProvider};
use tracing::info;

/// The initial device name when logging in with a device for the first time.
const INITIAL_DEVICE_DISPLAY_NAME: &str = "matrix-tui login client";

#[derive(Clone, Debug)]
pub enum LoginCredentials {
    Password {
        username: String,
        password: String,
    },
    #[allow(dead_code)]
    Other,
}

#[derive(Clone, Debug)]
pub enum LoginChoice {
    /// Login with username and password.
    Password,

    /// Login with SSO (Single Sign On).
    Sso,

    /// Login with a specific SSO identity provider.
    SsoIdp(IdentityProvider),
}

impl LoginChoice {
    /// Login with this login choice.
    pub async fn login(
        &self,
        client: &Client,
        credentials: Option<LoginCredentials>,
    ) -> Result<()> {
        match self {
            Self::Password => {
                let credentials = credentials
                    .context("Login with password was not provided username and/or password")?;
                login_with_password(client, credentials).await
            }
            Self::Sso => login_with_sso(client, None).await,
            Self::SsoIdp(idp) => login_with_sso(client, Some(idp)).await,
        }
    }
}

impl fmt::Display for LoginChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Password => write!(f, "Username and password"),
            Self::Sso => write!(f, "SSO"),
            Self::SsoIdp(idp) => write!(f, "SSO via {}", idp.name),
        }
    }
}

async fn login_with_password(client: &Client, credentials: LoginCredentials) -> Result<()> {
    info!("Logging in with username and password...");

    let LoginCredentials::Password { username, password } = credentials else {
        bail!("Login with password method was somehow provided the wrong type of credentials");
    };

    let username = username.trim();
    let password = password.trim();

    let matrix_auth = client.matrix_auth();

    matrix_auth
        .login_username(username, password)
        .initial_device_display_name(INITIAL_DEVICE_DISPLAY_NAME)
        .await?;

    Ok(())
}

async fn login_with_sso(client: &Client, idp: Option<&IdentityProvider>) -> Result<()> {
    info!("Logging in with SSO...");

    let matrix_auth = client.matrix_auth();
    let mut login_builder = matrix_auth.login_sso(|url| async move {
        // TODO: Have a crate open this URL in a browser
        info!("\nOpen this URL in your browser: {url}\n");
        info!("Waiting for login token...");

        Ok(())
    });

    if let Some(idp) = idp {
        login_builder = login_builder.identity_provider_id(&idp.id);
    }

    login_builder.await?;

    Ok(())
}
