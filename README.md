# Spotify Playlist Update - Eska Gorąca 20

A Rust application that automatically syncs a Spotify playlist with songs from Eska Gorąca 20 (Polish radio station hot list).

## Features

- 🎵 **Web Scraping**: Automatically scrapes the current songs from [Eska Gorąca 20](https://www.eska.pl/goraca20/)
- 🔍 **Spotify Search**: Searches for each song on Spotify using artist and track information
- 📝 **Playlist Management**: Updates an existing Spotify playlist by:
  - Removing all current songs
  - Adding the new songs from Gorąca 20 list
- 💾 **Smart Caching**: Caches search results to avoid redundant API calls
- 🔐 **OAuth Authentication**: Secure Spotify authentication with token persistence

## Prerequisites

- Rust 1.70 or higher
- Spotify Developer Account
- Spotify Premium Account (for playlist modification)

## Setup

### 1. Create Spotify App

1. Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
2. Create a new application
3. Add `http://localhost:8888/callback` as a Redirect URI
4. Copy your Client ID and Client Secret

### 2. Configure Environment Variables

1. Copy the `.env.dist` file to `.env`:
   ```bash
   cp .env.dist .env
   ```

2. Edit `.env` and add your Spotify credentials:
   ```
   CLIENT_ID=your_spotify_client_id
   CLIENT_SECRET=your_spotify_client_secret
   ```

### 3. Build the Project

```bash
cargo build --release
```

## Usage

### Running the Application

```bash
cargo run
```

On first run:
1. The application will open your browser for Spotify authentication
2. After authorizing, copy the redirect URL from your browser
3. Paste it into the terminal when prompted
4. The authentication token will be cached for future use

### What It Does

1. **Scrapes Eska Gorąca 20**: Fetches the current songs from the Gorąca 20 list
2. **Searches on Spotify**: For each song, searches Spotify using the artist and title
3. **Updates Playlist**: 
   - Finds your playlist named "Eska Gorąca" (create it manually first)
   - Removes all existing songs
   - Adds the new songs from the Gorąca 20 list
4. **Reports Results**: Shows which songs were found and which couldn't be matched

## Project Structure

```
spotify-playlist-update/
├── src/
│   ├── main.rs           # Main application logic and flow
│   ├── spotify_auth.rs   # Spotify OAuth authentication handling
│   ├── spotify.rs        # Spotify API operations (search, playlist management)
│   └── web_scraper.rs    # Web scraping for Eska Gorąca 20
├── .env.dist             # Environment variables template
├── Cargo.toml            # Rust dependencies
└── README.md             # This file
```

## Configuration

### Changing the Playlist Name

Edit line 53 in `src/main.rs`:
```rust
let playlist_name = "Eska Gorąca";  // Change to your preferred playlist name
```

### Changing the Source URL

Edit line 35 in `src/main.rs`:
```rust
let url = "https://www.eska.pl/goraca20/";  // Change to another chart if needed
```

## Dependencies

- **rspotify**: Spotify Web API client
- **tokio**: Async runtime
- **reqwest**: HTTP client for web scraping
- **scraper**: HTML parsing
- **dotenv**: Environment variable management
- **serde/serde_json**: JSON serialization for caching

## Cache Files

The application creates two cache files:
- `spotify_cache/.spotify_token_cache.json`: Spotify authentication token
- `cache.json`: Song search results cache

## Troubleshooting

### Songs Not Found
Some songs might not be found due to:
- Different spelling/formatting between Eska and Spotify
- Songs not available in your region
- New releases not yet on Spotify

### Authentication Issues
- Delete `spotify_cache/` directory to reset authentication
- Ensure redirect URI matches exactly: `http://localhost:8888/callback`
- Verify your Spotify app credentials are correct

### Playlist Not Found
- Create a playlist with the exact name "Eska Gorąca" (or your configured name)
- Ensure the playlist is in your own library

## License

This project is for personal use. Please respect Spotify's Terms of Service and API usage guidelines.

## Contributing

Feel free to submit issues and enhancement requests!