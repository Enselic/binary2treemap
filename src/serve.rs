use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;

use crate::TreemapData;

pub fn serve(treemap_data: &TreemapData) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(serve_impl(treemap_data));
}

#[derive(Debug, Clone)]
struct AppState<'a> {
    treemap_data: &'a TreemapData,
}

async fn serve_impl(treemap_data: &TreemapData) {
    let state = AppState { treemap_data };

    // build our application with a route
    let app = axum::Router::new()
        .route(
            "/d3.v7.min.js",
            get(Html(include_str!("../static/d3.v7.min.js").to_owned())),
        )
        .route("/*", get(treemap_page))
        .with_state(state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn treemap_page(State(state): State<AppState<'_>>, Path(path): Path<String>) -> String {
    format!("Hello, World! {} {}", state.treemap_data.size, path)
}
