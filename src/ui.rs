use handlebars::Handlebars;
use std::io::{BufRead, Result};
use std::path::PathBuf;
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
    fn for_path(&'d self, path: &Option<String>) -> Option<&'d TreemapNode> {
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

#[derive(askama::Template)]
#[template(path = "sourcefile.html")]
struct SourceFileTemplate<'a> {
    sorted_line_data: Vec<(&'a str, &'a str)>,
}

#[derive(Debug, Clone)]
struct UiState {
    treemap_data: Arc<TreemapNode>,
}

fn is_regular_file_and_can_be_opened(path: &str) -> bool {
    if let Ok(true) = std::fs::metadata(path).map(|metadata| metadata.is_file()) {
        std::fs::File::open(path).is_ok()
    } else {
        false
    }
}

async fn serve_impl(treemap_data: TreemapNode) -> Result<()> {
    let app = axum::Router::new()
        .route("/__debug__", get(debug_treemap_data))
        .route("/__data__/", get(data_handler))
        .route("/__data__/*key", get(data_handler))
        .route("/", get(treemap_or_file_handler))
        .route("/*key", get(treemap_or_file_handler))
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

async fn treemap_or_file_handler(
    State(state): State<UiState>,
    path: Option<Path<String>>,
) -> Html<String> {
    // TODO: Cache.
    let mut handlebars = Handlebars::new();

    let original_abs_path = path.map(|p| p.0).unwrap_or_default();

    // TODO: Add arg
    let abs_path = original_abs_path.replace(
        "/rustc/fb5ed726f72c6d16c788517c60ec00d4564b9348",
        "/home/martin/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust",
    );

    println!(
        "Looking for local file system path {} (mapped from {})",
        abs_path, original_abs_path
    );

    // TODO: Ensure to not open wrong file
    if is_regular_file_and_can_be_opened(&abs_path) {
        println!("Loading file: {}", abs_path);
        let full_path = format!("/{}", abs_path);
        let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
        let themes = syntect::highlighting::ThemeSet::load_defaults().themes;
        let theme = themes.get("base16-ocean.dark").unwrap();

        let mut highlighter = HighlightFile::new(full_path, &syntax_set, theme).unwrap();
        let (mut output, bg) = start_highlighted_html_snippet(theme);

        let mut line = String::new();

        let file_data = state
            .treemap_data
            .for_path(&Some(original_abs_path.clone()))
            .unwrap()
            .clone();

        let line_to_bytes = match file_data {
            TreemapNode::File { line_to_bytes, .. } => line_to_bytes.clone(),
            _ => unreachable!(),
        };

        // line.push_str("<pre>Bytes contributed to binary by line:<pre>\n\n\n");

        let mut line_nbr = 0;
        while highlighter.reader.read_line(&mut line).unwrap() > 0 {
            {
                line_nbr += 1;

                output.push_str(
                    line_to_bytes
                        .get(&line_nbr)
                        .map(|v| *v)
                        .unwrap_or_default()
                        .to_string()
                        .as_str(),
                );
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
    } else {
        // TODO: Give tip about --map-path flag
    }

    assert!(handlebars.register_template_string("index", source).is_ok());
    Html(
        handlebars
            .render("index", &HbsData { path: abs_path })
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
    Json(
        state
            .treemap_data
            .for_path(&path.map(|x| x.0))
            .unwrap()
            .clone(),
    )
}

async fn debug_treemap_data(State(state): State<UiState>) -> Html<String> {
    Html(format!("<pre>{state:#?}</pre>"))
}
