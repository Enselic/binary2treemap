use std::io::{BufRead, Result};
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;
use axum::Json;
use syntect::easy::HighlightFile;
use syntect::html::{
    append_highlighted_html_for_styled_line, start_highlighted_html_snippet, IncludeBackground,
};

use crate::TreemapNode;

pub fn serve(treemap_data: TreemapNode) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    rt.block_on(serve_impl(treemap_data))
}

impl<'d> TreemapNode {
    fn for_path(&'d self, path: &Option<Path<String>>) -> Option<&'d TreemapNode> {
        let path = match path {
            Some(path) => path.as_str(),
            None => return Some(self),
        };

        let mut current = self;
        for component in path.split('/') {
            if component.is_empty() {
                continue;
            }
            match current {
                TreemapNode::Directory { children, .. } => {
                    current = match children.get(component) {
                        Some(child) => child,
                        None => return None,
                    };
                }
                _ => break,
            }
        }
        Some(current)
    }
}

#[derive(Debug, Clone)]
struct UiState {
    treemap_data: Arc<TreemapNode>,
}

async fn serve_impl(treemap_data: TreemapNode) -> Result<()> {
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
    

    if matches!(
        path.as_ref().map(|p| p.0.as_str()),
        Some("home/martin/src/binary2treemap/src/ui.rs")
    ) {
        let full_path = format!("/{}", path.unwrap().0);
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        let themes = syntect::highlighting::ThemeSet::load_defaults().themes;
        let theme = themes.get("base16-ocean.dark").unwrap();

        let mut highlighter = HighlightFile::new(full_path, &syntax_set, theme).unwrap();
        let (mut output, bg) = start_highlighted_html_snippet(theme);

        let mut line = String::new();

        line.push_str("<pre>Bytes contributed to binary by line:<pre>\n\n\n");

        let mut line_nbr = 0;
        while highlighter.reader.read_line(&mut line).unwrap() > 0 {
            {
                line_nbr += 1;

                output.push_str(line_nbr.to_string().as_str());
                output.push_str("       ");
                let regions = highlighter
                    .highlight_lines
                    .highlight_line(&line, &syntax_set)
                    .unwrap();
                append_highlighted_html_for_styled_line(
                    &regions[..],
                    IncludeBackground::IfDifferent(bg),
                    &mut output,
                )
                .unwrap();
            }
            line.clear();
        }
        output.push_str("</pre>\n");
        return Html(output);
    }

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
) -> Json<TreemapNode> {
    Json(state.treemap_data.for_path(&path).unwrap().clone())
}

async fn debug_treemap_data(State(state): State<UiState>) -> Html<String> {
    Html(format!("<pre>{state:#?}</pre>"))
}
