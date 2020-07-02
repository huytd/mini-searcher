mod utils;
mod indexer;

use std::sync::RwLock;
use std::sync::Arc;
use tokio::runtime::Runtime;
use indexer::{SiteDefinition, Site};
use actix_web::{web, App, HttpServer, HttpResponse};

const REINDEX_INTERVAL: u64 = 60 * 60;

#[derive(serde::Deserialize)]
struct SearchParam {
    keyword: String
}

async fn index(site: web::Data<Arc<RwLock<Site>>>, req: web::Query<SearchParam>) -> HttpResponse {
    let pattern = &req.keyword;
    let result = {
        let s = site.read().unwrap();
        let articles = s.articles.to_owned();
        utils::search_site(articles, pattern)
    };
    HttpResponse::Ok().json(result)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let base_url = std::env::var("BASE_URL").expect("You must specify your site's BASE_URL.");
    let entry_point = std::env::var("ENTRY_POINT").expect("You must specify your site's ENTRY_POINT.");
    let search_pattern = std::env::var("LINK_SEARCH_PATTERN").expect("You must specify LINK_SEARCH_PATTERN.");
    let main_content_pattern = std::env::var("MAIN_CONTENT_PATTERN").expect("You must specify MAIN_CONTENT_PATTERN.");

    let site_def = SiteDefinition {
        base_url: base_url,
        entry_point: entry_point,
        search_pattern: search_pattern,
        main_content_pattern: main_content_pattern
    };

    let site = Site::from(site_def);
    let mutex = std::sync::RwLock::new(site);
    let arc = std::sync::Arc::new(mutex);

    let write_arc = arc.clone();
    std::thread::spawn(move || {
        loop {
            println!("Indexing...");
            {
                let mut guard = write_arc.write().unwrap();
                Runtime::new().unwrap().block_on((*guard).indexing());
            }
            println!("Done!");
            std::thread::sleep(std::time::Duration::from_secs(REINDEX_INTERVAL));
        }
    });

    let web_data = web::Data::new(arc.clone());
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let port = std::env::var("PORT").unwrap_or("3366".to_owned()).parse::<u16>().unwrap_or(3366);

    HttpServer::new(move || {
        App::new()
            .app_data(web_data.clone())
            .service(web::resource("/").to(index))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
