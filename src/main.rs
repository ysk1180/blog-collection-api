// 参考: https://zenn.dev/kowaremonoid/articles/7e077f9eb4439b

use axum::{routing::get, Router, Json};
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

#[derive(Serialize)]
struct ItemList(Vec<FeedItem>);

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

// TODO: 引数の型で配列の要素数を入れなきゃいけないのはなんかおかしいから直すす
fn fetch_feeds(urls: [&str; 3]) -> Vec<FeedItem> {
    let mut items = vec![];
    urls.iter().for_each(|url| {
        let mut fetch_items = fetch_feed(url);
        items.append(&mut fetch_items);
    });
    items
}

async fn root() -> axum::extract::Json<ItemList> {
    let urls = [
        "https://ysk-pro.hatenablog.com/rss",
        "https://blog.otegal.dev/rss.xml",
        "https://zenn.dev/yagince/feed",
    ];
    let mut items = fetch_feeds(urls);
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

    // jsonでレスポンスを返せるようにした（参考: https://github.com/joelparkerhenderson/demo-rust-axum#respond-with-a-json-payload ）
    Json(ItemList(items))
}
