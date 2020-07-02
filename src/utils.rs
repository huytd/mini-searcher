use crate::indexer::SearchResult;
use crate::indexer::Article;
use boyer_moore_magiclen::{BMByteSearchable, BMByteBadCharShiftMap};

const EXCERPT_LENGTH: usize = 250;
const EXCERPT_MARGIN: usize = 25;

pub fn search<TT: BMByteSearchable, TP: BMByteSearchable>(text: TT, pattern: TP) -> Vec<usize> {
    let bad_char_shift_map = BMByteBadCharShiftMap::create_bad_char_shift_map(&pattern).unwrap();
    boyer_moore_magiclen::byte::find_full(text, pattern, &bad_char_shift_map, 0)
}

pub fn take_excerpt(text: &str, at: usize) -> (String, String) {
    let right = &text[at..].chars().take(EXCERPT_LENGTH).collect::<String>();
    let rev_left = &text[..at].chars().rev().take(EXCERPT_MARGIN).collect::<String>();
    let left = &rev_left.chars().rev().collect::<String>();
    return (left.to_owned(), right.to_owned());
}

pub async fn fetch_text(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let result = reqwest::get(url).await?.text().await?;
    Ok(result)
}

pub fn search_site(articles: Vec<Article>, pattern: &str) -> Vec<SearchResult> {
    let mut result: Vec<SearchResult> = vec![];
    for article in articles {
        let founds = search(&article.content, pattern);
        if let Some(found) = founds.first() {
            let start = *found;
            let (left, right) = take_excerpt(&article.content, start);
            let excerpt = format!("...{}{}...", left, right);
            result.push(SearchResult {
                title: article.title,
                url: article.url,
                excerpt: excerpt.to_owned()
            })
        }
    }
    result
}
