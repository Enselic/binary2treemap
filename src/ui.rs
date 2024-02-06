use std::io::Result;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;
use axum::Json;
use handlebars::no_escape;

use crate::TreemapData;

/// `curl -L https://d3js.org/d3.v7.js -o vendor/d3.v7.js`
const D3_JS: Html<&'static str> = Html(include_str!("../vendor/d3.v7.js"));

pub fn serve(treemap_data: TreemapData) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    rt.block_on(serve_impl(treemap_data))
}

impl<'d> TreemapData {
    fn for_path(&'d self, path: &Option<Path<String>>) -> Option<&'d TreemapData> {
        let path = match path {
            Some(path) => path.as_str(),
            None => return Some(self),
        };

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

#[derive(Debug, Clone)]
struct UiState {
    treemap_data: Arc<TreemapData>,
}

async fn serve_impl(treemap_data: TreemapData) -> Result<()> {
    let app = axum::Router::new()
        .route("/__debug__", get(debug_treemap_data))
        .route("/__vendor__/d3.v7.js", get(D3_JS))
        .route("/__data__/", get(data_handler))
        .route("/__data__/*key", get(data_handler))
        .route("/", get(page_handler))
        .route("/*key", get(page_handler))
        .with_state(UiState {
            treemap_data: Arc::new(treemap_data),
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on http://{}", listener.local_addr()?);
    Ok(axum::serve(listener, app).await?)
}

#[derive(serde_derive::Serialize)]
struct HbsData {
    data: String,
}

async fn data_handler(State(state): State<UiState>, path: Option<Path<String>>) -> Json<String> {
    Json(serde_json::to_string(&state.treemap_data.for_path(&path)).unwrap())
}

async fn page_handler(State(state): State<UiState>, path: Option<Path<String>>) -> Html<String> {
    use handlebars::Handlebars;
    // TODO: Cache.
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);

    let source = include_str!("../static/index.hbs");
    assert!(handlebars.register_template_string("index", source).is_ok());
    if let Some(data) = state.treemap_data.for_path(&path) {
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

async fn debug_treemap_data(State(state): State<UiState>) -> Html<String> {
    Html(format!("<pre>{state:#?}</pre>"))
}
