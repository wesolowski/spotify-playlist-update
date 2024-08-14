use std::{env, fs};
use std::io::stdin;
use std::path::PathBuf;
use dotenv::dotenv;
use url::Url;
use rspotify::prelude::OAuthClient;
use rspotify::{scopes, AuthCodeSpotify, ClientError, Credentials, OAuth};

pub async fn get_spotify_client() -> Result<AuthCodeSpotify, ClientError> {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not set");
    let client_secret_id = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set");

    let scopes = scopes!(
        "playlist-read-private",
        "playlist-modify-public",
        "playlist-modify-private"
    );

    let mut oauth = OAuth::default();
    oauth.scopes = scopes;
    oauth.redirect_uri = "http://localhost:8888/callback".to_owned();

    let creds = Credentials::new(client_id.as_str(), client_secret_id.as_str());

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("spotify_cache");

    fs::create_dir_all(&path).unwrap();

    let config = rspotify::Config {
        token_cached: true,
        token_refreshing: true,
        cache_path: path.join(".spotify_token_cache.json"),
        ..Default::default()
    };

    let mut spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    match spotify.read_token_cache(true).await {
        Ok(Some(token)) => {
            *spotify.token.lock().await.unwrap() = Some(token);
        }
        Ok(None) => {
            handle_authorization_flow(&mut spotify).await?;
        }
        Err(e) => {
            println!("Failed to read token cache: {}", e);
            handle_authorization_flow(&mut spotify).await?;
        }
    }

    Ok(spotify)
}

async fn handle_authorization_flow(spotify: &mut AuthCodeSpotify) -> Result<(), ClientError> {
    let auth_url = spotify.get_authorize_url(true).unwrap();

    if webbrowser::open(&auth_url).is_err() {
        println!(
            "Failed to open the authorization URL. Please visit the URL manually: {}",
            auth_url
        );
    }

    println!("Enter redirected url:");
    let mut url_input = String::new();
    stdin().read_line(&mut url_input).unwrap();
    let url_string = &url_input.as_str();

    let url = Url::parse(url_string).expect("Failed to parse URL");
    let query_pairs = url.query_pairs();

    let mut code = String::new();
    let mut _state = String::new();
    for (key, value) in query_pairs {
        if key == "code" {
            code = value.to_string();
        } else if key == "state" {
            _state = value.to_string();
        }
    }

    spotify.request_token(code.trim()).await?;

    Ok(())
}

