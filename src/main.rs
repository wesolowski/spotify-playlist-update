use std::collections::HashMap;
use rspotify::prelude::OAuthClient;
use rspotify::{scopes, AuthCodeSpotify, ClientError, Credentials, OAuth};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, stdin};
use std::path::PathBuf;
use rspotify::model::{PlayableId, SearchResult, SearchType, SimplifiedPlaylist, TrackId};
use url::Url;
use webbrowser;
use thiserror::Error;
use dotenv::dotenv;
use rspotify::clients::BaseClient;
use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Cache {
    songs: HashMap<String, (TrackId<'static>, (String, String))>,
}

#[derive(Debug, Error)]
pub enum SpotifyError {
    #[error("Playlist '{0}' not found")]
    PlaylistNotFound(String),
    #[error("Spotify API error: {0}")]
    SpotifyApiError(#[from] rspotify::ClientError),
}

#[derive(Debug, Clone)]
struct SongQuery {
    artist: String,
    title: String,
}



#[tokio::main]
async fn main() {

    let url = "https://www.eska.pl/goraca20/";
    let response = reqwest::get(url).await.unwrap().text().await.unwrap();

    // HTML parsen
    let document = Html::parse_document(&response);
    let selector = Selector::parse("div.single-hit").unwrap();

    let mut songs = Vec::new();

    for element in document.select(&selector) {
        let title_selector = Selector::parse("a.single-hit__title").unwrap();
        let author_selector = Selector::parse("a.single-hit__author").unwrap();

        let title = element
            .select(&title_selector)
            .next()
            .map(|e| e.inner_html())
            .unwrap_or_else(|| "".to_string());

        let mut artist = element
            .select(&author_selector)
            .next()
            .map(|e| e.inner_html())
            .unwrap_or_else(|| "".to_string());

        if !artist.is_empty() && !title.is_empty() {

            artist = artist.replace("&amp;", "&");

            let song = SongQuery { artist, title };
            songs.push(song);
        }
    }

    for song in &songs {
        println!("{:?}", song);
    }


    let spotify = get_spotify_client().await.unwrap();



    let (song_results, not_found_queries) = search_songs(&spotify, &songs).await;


    // for (id, (name, artists)) in song_results {
    //     println!("ID: {}, Name: {}, Artists: {}", id, name, artists);
    // }

    if !not_found_queries.is_empty() {
        println!("Songs not found:");
        for query in not_found_queries {
            println!("Artist: {}, Title: {}", query.artist, query.title);
        }
    }


    //let playlist_name = "Esk@ Gorąca Lista";
    let playlist_name = "RustTest";
    let playlist = get_playlist_by_name(&spotify, playlist_name).await.unwrap();



    println!("Playlist: {}", playlist.name);

    let mut track_ids = Vec::new();

    let limit = 50;
    let mut offset = 0;
    println!("Items:");
    loop {


        let page = spotify
            .playlist_items_manual(playlist.id.clone(), None, None, Some(limit), Some(offset))
            .await
            .unwrap();

        for item in page.items {
            if let Some(playable_item) = item.track {
                match playable_item {
                    rspotify::model::PlayableItem::Track(track) => {
                        track_ids.push(track.id.clone());
                        let track_name = track.id.unwrap();
                        let album_name = track.name;
                        println!("Track: {}, Album: {}", track_name, album_name);
                    },
                    _ => println!("Not a track"),
                }
            } else {
                println!("Item has no track");
            }
        }



        if page.next.is_none() {
            break;
        }

        offset += limit;
    }


    for chunk in track_ids.chunks(50) {
        let playable_ids: Vec<PlayableId> = chunk
            .iter()
            .filter_map(|id| id.clone())
            .map(|id| PlayableId::from(id))
            .collect();

        println!("Playable IDs batch size: {:?}", playable_ids.len());

        let result = spotify
            .playlist_remove_all_occurrences_of_items(
                playlist.id.clone(),
                playable_ids,
                None,
            )
            .await;

        match result {
            Ok(response) => println!("Removed tracks from playlist: {:?}", response),
            Err(err) => eprintln!("Failed to remove tracks: {:?}", err),
        }
    }


    for chunk in song_results.chunks(50) {
        let playable_ids: Vec<PlayableId> = chunk
            .iter()
            .filter_map(|(id, _)| Some(PlayableId::from(id.clone())))
            .collect();

        println!("Playable IDs batch size: {:?}", playable_ids.len());

        let result = spotify
            .playlist_add_items(
                playlist.id.clone(),
                playable_ids,
                None,
            )
            .await;

        match result {
            Ok(response) => println!("Add tracks to playlist: {:?}", response),
            Err(err) => eprintln!("Failed add tracks: {:?}", err),
        }
    }

    return;


}

async fn get_playlist_by_name(spotify: &AuthCodeSpotify, playlist_name: &str) -> Result<SimplifiedPlaylist, SpotifyError> {
    let mut offset = 0;

    let limit = 50;

    loop {
        let playlists_result = spotify.current_user_playlists_manual(Some(limit), Some(offset)).await.unwrap();

        for playlist in playlists_result.items {
            if playlist.name == playlist_name {
                return Ok(playlist);
            }
        }

        match playlists_result.next {
            Some(next_url) => {
                let url = Url::parse(&next_url).unwrap();
                let query_pairs = url.query_pairs().into_owned().collect::<HashMap<String, String>>();
                if let Some(offset_str) = query_pairs.get("offset") {
                    offset = offset_str.parse::<usize>().unwrap() as u32;
                } else {
                    break;
                }
            },
            None => break,
        }
    }

    Err(SpotifyError::PlaylistNotFound(playlist_name.to_string()))
}

pub async fn get_spotify_client() -> Result<AuthCodeSpotify, ClientError> {
    dotenv().ok();

    // let client_id = "6eb950cfd0ef453dbfb0bd859ffa208c";
    // let client_secret_id = "2778773674a04bb6b8db3251f1c518c1";

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not set");
    let client_secret_id = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set");

    // Defining the scopes (permissions) required for the application
    let scopes = scopes!(
        "user-read-email",
        "user-read-private",
        "user-top-read",
        "user-read-recently-played",
        "user-follow-read",
        "user-library-read",
        "user-read-currently-playing",
        "user-read-playback-state",
        "user-read-playback-position",
        "playlist-read-collaborative",
        "playlist-read-private",
        "user-follow-modify",
        "user-library-modify",
        "user-modify-playback-state",
        "playlist-modify-public",
        "playlist-modify-private",
        "ugc-image-upload"
    );

    let mut oauth = OAuth::default();
    oauth.scopes = scopes;
    oauth.redirect_uri = "http://localhost:8888/callback".to_owned();

    let creds = Credentials::new(client_id.as_str(), client_secret_id.as_str());

    // print env!("CARGO_MANIFEST_DIR")

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
            // No cached token found, enter the authorization flow
            handle_authorization_flow(&mut spotify).await?;
        }
        Err(e) => {
            println!("Failed to read token cache: {}", e);
            // Handle the error, e.g., by entering the authorization flow
            handle_authorization_flow(&mut spotify).await?;
        }
    }

    Ok(spotify)
}

// Function to handle the authorization flow with Spotify
async fn handle_authorization_flow(spotify: &mut AuthCodeSpotify) -> Result<(), ClientError> {
    let auth_url = spotify.get_authorize_url(true).unwrap(); // Getting the authorization URL

    if webbrowser::open(&auth_url).is_err() {
        // Attempting to open the authorization URL in the default browser
        println!(
            "Failed to open the authorization URL. Please visit the URL manually: {}",
            auth_url
        );
    }

    // Prompting the user to enter the redirected URL after authorization
    println!("Enter redirected url:");
    let mut url_input = String::new();
    stdin().read_line(&mut url_input).unwrap();
    let url_string = &url_input.as_str();

    // Parsing the redirected URL
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

    // Requesting the access token using the authorization code
    spotify.request_token(code.trim()).await?;

    Ok(())
}

async fn search_songs(spotify: &AuthCodeSpotify, queries: &[SongQuery]) -> (Vec<(TrackId<'static>, (String, String))>, Vec<SongQuery>) {
    let mut results = Vec::new();
    let mut not_found = Vec::new();

    let cache_file = "cache.json";
    let cache: Cache = if let Ok(file) = File::open(cache_file) {
        serde_json::from_reader(BufReader::new(file)).unwrap_or_else(|_| Cache { songs: HashMap::new() })
    } else {
        Cache { songs: HashMap::new() }
    };

    let mut updated_cache = cache.songs.clone();

    for query in queries {
        let search_query = format!("artist:{} track:{}", query.artist, query.title);
        if let Some(cached_result) = cache.songs.get(&search_query) {
            results.push(cached_result.clone());
        } else {
            match spotify.search(&search_query, SearchType::Track, None, None, Some(1), None).await {
                Ok(SearchResult::Tracks(tracks)) => {
                    if let Some(track) = tracks.items.first() {
                        if let Some(track_id) = &track.id {
                            let result = (
                                track_id.clone(),
                                (track.name.clone(), track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")),
                            );
                            results.push(result.clone());
                            updated_cache.insert(search_query.clone(), result);
                        }
                    } else {
                        not_found.push(query.clone());
                    }
                },
                Err(..) => {
                    not_found.push(query.clone());
                },
                _ => {
                    not_found.push(query.clone());
                },
            }
        }
    }

    let file = File::create(cache_file).unwrap();
    serde_json::to_writer(BufWriter::new(file), &Cache { songs: updated_cache }).unwrap();

    (results, not_found)
}
