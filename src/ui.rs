use std::io::Result;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;
use handlebars::no_escape;

use crate::TreemapData;

pub fn serve(treemap_data: TreemapData) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    rt.block_on(serve_impl(treemap_data))
}

impl<'d> TreemapData {
    fn for_path(&'d self, path: &Path<String>) -> Option<&'d TreemapData> {
        let mut current = self;
        for component in path.split('/') {
            if component.is_empty() {
                continue;
            }
            current = match current.children.get(component) {
                Some(child) => child,
                None => return None,
            };
        }
        Some(current)
    }
}

async fn serve_impl(treemap_data: TreemapData) -> Result<()> {
    // build our application with a route
    let app = axum::Router::new()
        .route(
            "/d3.v7.min.js",
            get(Html(include_str!("../static/d3.v7.min.js").to_owned())),
        )
        .route("/*key", get(treemap_page))
        .with_state(Arc::new(treemap_data));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on http://{}", listener.local_addr()?);
    Ok(axum::serve(listener, app).await?)
}

async fn treemap_page(State(state): State<Arc<TreemapData>>, path: Path<String>) -> Html<String> {
    use handlebars::Handlebars;
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);

    // register the template. The template string will be verified and compiled.
    let source = include_str!("../static/index.hbs");
    assert!(handlebars.register_template_string("index", source).is_ok());
    if let Some(data) = state.for_path(&path) {
        #[derive(serde_derive::Serialize)]
        struct HbsData {
            data: String,
        }
        eprintln!("NORDH1 {data:#?} NORDH2");
        Html(
            handlebars
                .render(
                    "index",
                    &HbsData {
                        data: serde_json::to_string(data).unwrap(),
                    },
                )
                .unwrap(),
        )
    } else {
        Html(format!(
            "ERROR: Could not find {path:?} in <pre>{state:#?}</pre>"
        ))
    }
}
