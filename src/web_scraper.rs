use scraper::{Html, Selector};
use crate::SongQuery;

pub async fn scrape_songs(url: &str) -> Vec<SongQuery> {
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
    songs
}