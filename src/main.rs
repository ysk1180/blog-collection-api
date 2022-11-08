// 参考: https://zenn.dev/kowaremonoid/articles/7e077f9eb4439b

use axum::{routing::get, Router};
use chrono::{DateTime};
use rss::Channel;
use serde::Serialize;
use std::{net::SocketAddr, cmp::Ordering};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Debug)]
struct FeedItem {
    title: String,
    link: String,
    pub_date: String,
}

// 参考：https://blog-dry.com/entry/2018/03/21/225533
fn fetch_feed(url: &str) -> Vec<FeedItem> {
    let channel = Channel::from_url(url).unwrap();
    let items: Vec<FeedItem> = channel
        .items()
        .iter()
        .map(|item| FeedItem {
            title: item.title().unwrap().to_string(),
            link: item.link().unwrap().to_string(),
            pub_date: item.pub_date().unwrap().to_string(),
        })
        .collect();
    items
}

async fn root() -> &'static str {
    let url = "https://ysk-pro.hatenablog.com/rss";
    let mut items = fetch_feed(url);
    // 日付でのソート：https://shinshin86.hateblo.jp/entry/2022/03/26/060000
    items.sort_by(|a, b| {
        let duration = DateTime::parse_from_rfc2822(&*a.pub_date).unwrap() - DateTime::parse_from_rfc2822(&*b.pub_date).unwrap();
        if duration.num_milliseconds() > 0 {
            Ordering::Less
        } else if duration.num_milliseconds() == 0 {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });
    items.iter()
        .for_each(|item| println!("{}", serde_json::to_string(&item).unwrap()));
    "Hello, World!"
}
