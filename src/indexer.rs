use scraper::{Html, Selector};
use serde::{ Serialize, Deserialize };
use crate::utils;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteDefinition {
  pub base_url: String,
  pub entry_point: String,
  pub search_pattern: String,
  pub main_content_pattern: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
  pub url: String,
  pub title: String,
  pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
  pub url: String,
  pub title: String,
  pub excerpt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub site_definition: SiteDefinition,
    pub articles: Vec<Article>
}

impl Site {
    pub fn from(site_definition: SiteDefinition) -> Site {
        Site {
            site_definition: site_definition,
            articles: vec![]
        }
    }

    pub async fn indexing(&mut self) {
        let mut db: Vec<Article> = vec![];

        if let Ok(html) = utils::fetch_text(&self.site_definition.entry_point).await {
            let document = Html::parse_document(html.as_str());
            let links_selector = Selector::parse(&self.site_definition.search_pattern).unwrap();
            let links = document.select(&links_selector);

            for link in links {
                let title = link.text().collect::<Vec<_>>().join("");
                if let Some(url) = link.value().attr("href") {
                    let page_url = self.site_definition.base_url.to_owned() + url;
                    if let Ok(page_content) = utils::fetch_text(&page_url).await {
                        let document = Html::parse_document(page_content.as_str());
                        let find_content = Selector::parse(&self.site_definition.main_content_pattern).unwrap();
                        let matched_contents = document.root_element().select(&find_content);
                        let mut extracted_content: Vec<String> = vec![];
                        for content in matched_contents {
                            let inner_text = content.text().filter(|s| *s != "\n").collect::<Vec<_>>().join("");
                            extracted_content.push(inner_text);
                        }
                        let full_content = extracted_content.join(" ");

                        println!("{} {}", url, title);

                        db.push(Article {
                            url: url.to_owned(),
                            title: title,
                            content: full_content
                        });
                    }
                }
            }
        }

        self.articles = db;
    }
}
