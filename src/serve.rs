use axum::routing::get;

pub fn serve(data: String) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(serve_impl(data));
}

async fn serve_impl(data: String) {
    // build our application with a route
    let app = axum::Router::new()
        .route("/", get(include_str!("../static/index.html").to_owned()))
        .route(
            "/d3.v7.min.js",
            get(include_str!("../static/d3.v7.min.js").to_owned()),
        )
        .route("/data.json", get(data));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
