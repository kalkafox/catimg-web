use catimg_backend::Image;
use mongodb::bson::doc;
use mongodb::options::ClientOptions;
use rand::distributions::{Alphanumeric, DistString};
use sha3::Digest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::main;
use tokio::process::Command;
use warp::Filter;

#[main]
async fn main() {
    let playwright = Arc::new(playwright::Playwright::initialize()
        .await
        .unwrap());

    playwright.prepare().unwrap();

    let catimg_route = warp::path!("catimg")
        .and(warp::query::<HashMap<String, String>>())
        .and_then(|query: HashMap<String, String>| async move {
            // let redis_config = match std::env::var("UPSTASH_URL") {
            //     Ok(url) => {
            //         let config = redis::Client::open(url).unwrap();
            //         config.get_connection().unwrap();
            //         Some(config)
            //     },
            //     Err(_) => None,
            // };
            //
            // let mut redis_config = match redis_config {
            //     Some(config) => config,
            //     None => return Err(warp::reject::not_found()),
            // };

            let mongo_config = match std::env::var("MONGO_URL") {
                Ok(url) => {
                    let config = match ClientOptions::parse(url).await {
                        Ok(mut config) => {
                            config.app_name = Some("kalkafox-catimg".to_string());
                            config
                        }
                        Err(e) => {
                            eprintln!("Failed to parse MONGO_URL: {}", e);
                            return Err(warp::reject::not_found());
                        }
                    };
                    config
                }
                Err(e) => {
                    eprintln!("MONGO_URL not set: {}", e);
                    return Err(warp::reject::not_found());
                }
            };

            // let mut mongo_config = std::env::var("MONGO_URL");
            // if let Err(url) = mongo_config {
            //     eprintln!("MONGO_URL not set: {}", url);
            //     return Err(warp::reject::not_found());
            // }
            //
            // let mongo_client = ClientOptions::parse(mongo_config.unwrap()).await;
            //
            // if let Err(e) = mongo_client {
            //     eprintln!("Failed to parse MONGO_URL: {}", e);
            //     return Err(warp::reject::not_found());
            // }
            //
            // let mongo_client = mongo_client.unwrap();

            // let config = match ClientOptions::parse(mongo_config.unwrap()).await {
            //     Ok(mut config) => {
            //         config.app_name = Some("kalkafox-catimg".to_string());
            //         config
            //     },
            //     Err(e) => {
            //         eprintln!("Failed to parse MONGO_URL: {}", e);
            //         return Err(warp::reject::not_found());
            //     }
            // };

            let mongo_client = match mongodb::Client::with_options(mongo_config) {
                Ok(client) => client,
                Err(e) => {
                    eprintln!("Failed to connect to MongoDB: {}", e);
                    return Err(warp::reject::not_found());
                }
            };
            //
            // if let Some({/*can't get env here yet*/}) = mongo_client {
            // } else {
            //     eprintln!("Failed to connect to MongoDB");
            //     return Err(warp::reject::not_found());
            // }

            let width = match query.get("w") {
                Some(width) => width,
                None => "200",
            };

            let url = match query.get("url") {
                Some(url) => url,
                None => return Err(warp::reject::not_found()),
            };

            if url.is_empty() {
                return Err(warp::reject::not_found());
            }

            // Check if the url returns an image
            let mut response = reqwest::get(url).await.unwrap();

            let headers = response.headers();

            if !response.status().is_success() {
                return Err(warp::reject::not_found());
            }

            if !headers
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("image")
            {
                return Err(warp::reject::not_found());
            }

            let hash = sha3::Sha3_256::digest(format!("{}{}", url, width).as_bytes());

            // if let Ok(image) = redis_config.get::<_, String>(format!("{:x}", hash)) {
            //     return Ok::<_, warp::Rejection>(warp::reply::with_status(image, warp::http::StatusCode::OK));
            // }

            let catimg_db = mongo_client.database("catimg");

            let catimg_collection = catimg_db.collection::<Image>("images");

            let image = match catimg_collection
                .find_one(doc! {"id": format!("{:x}", hash)}, None)
                .await
            {
                Ok(image) => image,
                Err(e) => {
                    eprintln!("Failed to query MongoDB: {}", e);
                    return Err(warp::reject::not_found());
                }
            };

            if let Some(image) = image {
                println!("Found image in MongoDB: {}", image.id);
                return Ok::<_, warp::Rejection>(warp::reply::with_status(
                    image.data,
                    warp::http::StatusCode::OK,
                ));
            }

            let rand_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

            // Save to /tmp
            let mut write_file = tokio::fs::File::create(format!("/tmp/{}", rand_id))
                .await
                .unwrap();

            while let Some(chunk) = response.chunk().await.unwrap() {
                write_file.write_all(&chunk).await.unwrap();
            }

            let output = Command::new("catimg")
                .arg(format!("/tmp/{}", rand_id))
                .arg(format!("-w {}", width))
                .output()
                .await
                .unwrap();

            let stdout = String::from_utf8(output.stdout).unwrap();

            // Delete the file
            match tokio::fs::remove_file(format!("/tmp/{}", rand_id)).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to delete file: {}", e);
                }
            }

            // Convert hash to string
            let hash = format!("{:x}", hash);

            //let _: () = redis_config.set(hash, stdout).unwrap();

            let image = Image {
                id: hash.clone(),
                data: stdout.clone(),
            };

            match catimg_collection.insert_one(image, None).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to insert into MongoDB: {}", e);
                    return Err(warp::reject::not_found());
                }
            }

            //Ok::<_, warp::Rejection>(warp::reply::with_status(write_file.read_to_end(&mut Vec::new()).await.unwrap().to_string(), warp::http::StatusCode::OK))
            Ok::<_, warp::Rejection>(warp::reply::with_status(stdout, warp::http::StatusCode::OK))
        });

    let preview_route = warp::path!("preview")
        .and(warp::query::<HashMap<String, String>>())
        .and_then(|query: HashMap<String, String>| async move {
            let playwright = playwright::Playwright::initialize().await.unwrap();
            let chromium = playwright.chromium();
            let browser = chromium.launcher().headless(true).launch().await.unwrap();
            let context = browser.context_builder().build().await.unwrap();
            let page = context.new_page().await.unwrap();

            let prefix_url = match std::env::var("CATIMG_URL") {
                Ok(prefix_url) => prefix_url,
                Err(_) => "https://catimg.kalkafox.dev".to_string(),
            };

            let formatted_url = format!(
                "{}/?url={}&w={}&fs=true",
                prefix_url,
                query.get("url").unwrap(),
                query.get("w").unwrap_or(&"200".to_string())
            );

            page.goto_builder(formatted_url.as_str())
                .goto()
                .await
                .unwrap();

            // Continuously page.eval("() => window.terminalLoaded").await until it's true

            let now = std::time::Instant::now();

            loop {
                // If it's been more than 30 seconds, return 404
                if now.elapsed().as_secs() > 30 {
                    return Err(warp::reject::not_found());
                }
                // If terminalLoaded is true, break
                let loaded = page.eval("() => window.terminalLoaded").await;
                if let Ok(loaded) = loaded {
                    if loaded {
                        break;
                    }
                }
                // Otherwise, sleep for 100ms
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            let image = page.screenshot_builder().screenshot().await.unwrap();

            Ok::<_, warp::Rejection>(warp::reply::with_header(image, "Content-Type", "image/png"))
        });

    let assets_route = warp::path!("assets" / String).and_then(|file_name: String| async move {
        let path = match std::env::var("CATIMG_PATH") {
            Ok(path) => path,
            Err(_) => "/var/www/catimg".to_string(),
        };

        let file = tokio::fs::read(format!("{}/assets/{}", path, file_name))
            .await
            .unwrap();

        let mime_type = mime_guess::from_path(file_name).first_or_octet_stream();

        Ok::<_, warp::Rejection>(warp::reply::with_header(
            file,
            "Content-Type",
            mime_type.as_ref(),
        ))
    });

    let html_route = warp::path::end()
        .and(warp::query::<HashMap<String, String>>())
        .and_then(|query: HashMap<String, String>| async move {
            let path = match std::env::var("CATIMG_PATH") {
                Ok(path) => path,
                Err(_) => "/var/www/catimg".to_string(),
            };

            let file = tokio::fs::read_to_string(format!("{}/index.html", path))
                .await
                .unwrap();

            let prefix_url = match std::env::var("CATIMG_URL") {
                Ok(prefix_url) => prefix_url,
                Err(_) => "https://catimg.kalkafox.dev".to_string(),
            };

            let formatted_url = format!(
                "{}/preview?url={}&w={}&fs=true",
                prefix_url,
                query.get("url").unwrap_or(&"".to_string()),
                query.get("w").unwrap_or(&"200".to_string())
            );

            let file = file.replace(
                "<!-- OG:IMAGE -->",
                format!("<meta content={} property=\"og:image\" />", formatted_url).as_str(),
            );

            Ok::<_, warp::Rejection>(warp::reply::with_header(file, "Content-Type", "text/html"))
        });

    let routes = catimg_route
        .or(preview_route)
        .or(html_route)
        .or(assets_route)
        .with(warp::cors().allow_any_origin());

    println!("Listening on port 3030");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
