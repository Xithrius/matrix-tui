use std::{
    fmt,
    io::{self, Write},
};

use color_eyre::Result;
use matrix_sdk::{Client, ruma::api::client::session::get_login_types::v3::IdentityProvider};
use tracing::info;

/// The initial device name when logging in with a device for the first time.
const INITIAL_DEVICE_DISPLAY_NAME: &str = "matrix-tui login client";

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
    pub async fn login(&self, client: &Client) -> Result<()> {
        match self {
            LoginChoice::Password => login_with_password(client).await,
            LoginChoice::Sso => login_with_sso(client, None).await,
            LoginChoice::SsoIdp(idp) => login_with_sso(client, Some(idp)).await,
        }
    }
}

impl fmt::Display for LoginChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoginChoice::Password => write!(f, "Username and password"),
            LoginChoice::Sso => write!(f, "SSO"),
            LoginChoice::SsoIdp(idp) => write!(f, "SSO via {}", idp.name),
        }
    }
}

async fn login_with_password(client: &Client) -> Result<()> {
    println!("Logging in with username and password…");

    loop {
        print!("\nUsername: ");
        io::stdout().flush().expect("Unable to write to stdout");
        let mut username = String::new();
        io::stdin()
            .read_line(&mut username)
            .expect("Unable to read user input");
        username = username.trim().to_owned();

        print!("Password: ");
        io::stdout().flush().expect("Unable to write to stdout");
        let mut password = String::new();
        io::stdin()
            .read_line(&mut password)
            .expect("Unable to read user input");
        password = password.trim().to_owned();

        match client
            .matrix_auth()
            .login_username(&username, &password)
            .initial_device_display_name(INITIAL_DEVICE_DISPLAY_NAME)
            .await
        {
            Ok(_) => {
                println!("Logged in as {username}");
                break;
            }
            Err(error) => {
                println!("Error logging in: {error}");
                println!("Please try again\n");
            }
        }
    }

    Ok(())
}

async fn login_with_sso(client: &Client, idp: Option<&IdentityProvider>) -> Result<()> {
    info!("Logging in with SSO…");

    let mut login_builder = client.matrix_auth().login_sso(|url| async move {
        // TODO: Have a crate open this URL in a browser
        info!("\nOpen this URL in your browser: {url}\n");
        info!("Waiting for login token…");

        Ok(())
    });

    if let Some(idp) = idp {
        login_builder = login_builder.identity_provider_id(&idp.id);
    }

    login_builder.await?;

    info!("Logged in as {}", client.user_id().unwrap());

    Ok(())
}
