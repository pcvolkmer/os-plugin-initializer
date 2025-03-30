mod zip_file;

use crate::zip_file::ProjectFile;
use askama::Template;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::sync::LazyLock;
use std::{env, path};
use tracing::log::{error, info};

#[cfg(unix)]
use tokio::signal;
#[cfg(debug_assertions)]
use tower_http::trace::TraceLayer;

static ASSETS: Dir = include_dir!("src/resources/assets");

static OS_VERSIONS: LazyLock<Vec<String>> = LazyLock::new(|| {
    env::var("OS_VERSIONS")
        .unwrap_or_else(|_| "2.14.0,2.13.2".to_string())
        .split(",")
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
});

#[derive(Deserialize)]
struct Params {
    project_type: String,
    os_version: String,
    group: String,
    artifact: String,
    description: String,
    package_name: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    os_versions: Vec<String>,
}

async fn index() -> Result<impl IntoResponse, ()> {
    let rendered = IndexTemplate {
        os_versions: OS_VERSIONS
            .iter()
            .map(|version| version.to_string())
            .collect::<Vec<String>>(),
    }
    .render()
    .map_err(|_| ())?;

    Ok(Html(rendered))
}

async fn zip_package(params: Query<Params>) -> impl IntoResponse {
    let zip = ProjectFile::new(
        params.project_type.to_string(),
        if OS_VERSIONS.contains(&params.os_version) {
            params.os_version.to_string()
        } else {
            OS_VERSIONS.first().unwrap().to_string()
        },
        params.group.to_string(),
        params.artifact.to_string(),
        params.description.to_string(),
        params.package_name.to_string(),
    )
    .to_zip_archive()
    .unwrap()
    .into_inner()
    .into_inner();

    Response::builder().body(Body::from(zip)).unwrap()
}

async fn serve_asset(path: Option<Path<String>>) -> impl IntoResponse {
    fn get_mimetype(path: &path::Path) -> Option<&str> {
        if let Some(extension) = path.extension() {
            return match extension.to_str() {
                Some("css") => Some("text/css"),
                Some("js") => Some("application/javascript"),
                _ => None,
            };
        }
        None
    }

    match path {
        Some(path) => match ASSETS.get_file(path.to_string()) {
            Some(file) => {
                if let Some(mime_type) = get_mimetype(file.path()) {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, mime_type)
                        .body(Body::from(file.contents()))
                } else {
                    Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from(file.contents()))
                }
            }
            None => Response::builder()
                .status(404)
                .body(Body::from("".as_bytes())),
        },
        None => Response::builder()
            .status(400)
            .body(Body::from("".as_bytes())),
    }
    .unwrap()
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/project.zip", get(zip_package))
        .route(
            "/assets/{*path}",
            get(|path| async { serve_asset(path).await }),
        );

    #[cfg(debug_assertions)]
    let app = app.layer(TraceLayer::new_for_http());

    let listener_address = env::var("LISTENER_ADDRESS").unwrap_or_else(|_| "[::]:3000".to_string());

    match tokio::net::TcpListener::bind(listener_address).await {
        Ok(listener) => {
            let address = listener.local_addr().unwrap();
            if address.is_ipv6() {
                info!("Listening on [{}]:{}", address.ip(), address.port());
            } else {
                info!("Listening on {}:{}", address.ip(), address.port());
            }
            match axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
            {
                Ok(_) => {}
                Err(err) => error!("Unable to start application: {}", err),
            }
        }
        Err(err) => error!("Unable to bind server port: {}", err),
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    terminate.await;
}
