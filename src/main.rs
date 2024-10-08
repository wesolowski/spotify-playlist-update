mod web_scraper;
mod spotify;
mod spotify_auth;

use std::collections::HashMap;
use rspotify::model::{TrackId};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::spotify::{add_songs, delete_all_songs, get_playlist_by_name, search_songs};
use crate::spotify_auth::get_spotify_client;
use crate::web_scraper::scrape_songs;

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

    let songs = scrape_songs(url).await;

    let spotify = get_spotify_client().await.unwrap();

    let (song_results, not_found_queries) = search_songs(&spotify, &songs).await;

    if !not_found_queries.is_empty() {
        println!("Songs not found:");
        for query in not_found_queries {
            println!("Artist: {}, Title: {}", query.artist, query.title);
        }
    }

    let playlist_name = "Eska Gorąca";
    //let playlist_name = "RustTest";
    let playlist = get_playlist_by_name(&spotify, playlist_name).await.unwrap();

    delete_all_songs(&spotify, &playlist).await;

    add_songs(spotify, song_results, playlist).await;

    return;
}


