use std::io::Result;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;
use axum::Json;

use crate::TreemapData;

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
        .route("/__data__/", get(data_handler))
        .route("/__data__/*key", get(data_handler))
        .route("/", get(page_handler))
        .route("/*key", get(page_handler))
        .with_state(UiState {
            treemap_data: Arc::new(treemap_data),
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await
}

#[derive(serde_derive::Serialize)]
struct HbsData {
    path: String,
}

async fn page_handler(path: Option<Path<String>>) -> Html<String> {
    use handlebars::Handlebars;
    // TODO: Cache.
    let mut handlebars = Handlebars::new();

    let source = include_str!("../static/index.hbs");
    assert!(handlebars.register_template_string("index", source).is_ok());
    Html(
        handlebars
            .render(
                "index",
                &HbsData {
                    path: path.map(|p| p.0).unwrap_or_default(),
                },
            )
            .unwrap(),
    )
    // } else {
    //     Html(format!(
    //         "ERROR: Could not find {path:?}. Maybe you want to visit <a href=\"/__debug__\""
    //     ))
    // }
}

async fn data_handler(
    State(state): State<UiState>,
    path: Option<Path<String>>,
) -> Json<TreemapData> {
    Json(state.treemap_data.for_path(&path).unwrap().clone())
}

async fn debug_treemap_data(State(state): State<UiState>) -> Html<String> {
    Html(format!("<pre>{state:#?}</pre>"))
}
