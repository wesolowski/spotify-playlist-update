use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use rspotify::AuthCodeSpotify;
use rspotify::model::{PlayableId, SearchResult, SearchType, SimplifiedPlaylist, TrackId};
use url::Url;

use rspotify::prelude::OAuthClient;
use rspotify::clients::BaseClient;
use crate::{Cache, SongQuery, SpotifyError};

pub async fn add_songs(spotify: AuthCodeSpotify, song_results: Vec<(TrackId<'static>, (String, String))>, playlist: SimplifiedPlaylist) {
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
}

pub async fn delete_all_songs(spotify: &AuthCodeSpotify, playlist: &SimplifiedPlaylist) {
    let mut track_ids = Vec::new();

    let limit = 50;
    let mut offset = 0;

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
}



pub async fn get_playlist_by_name(spotify: &AuthCodeSpotify, playlist_name: &str) -> Result<SimplifiedPlaylist, SpotifyError> {
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


pub async fn search_songs(spotify: &AuthCodeSpotify, queries: &[SongQuery]) -> (Vec<(TrackId<'static>, (String, String))>, Vec<SongQuery>) {
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