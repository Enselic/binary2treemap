use axum::{response::Html, routing::get};

pub fn serve() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(serve_impl()); // You can now call async functions using block_on
}

async fn serve_impl() {
    // build our application with a route
    let app = axum::Router::new()
        .route("/", get(handler))
        .route("/d3.v7.min.js", get(d3));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn d3() -> Html<&'static str> {
    Html(include_str!("../static/d3.v7.min.js"))
}
