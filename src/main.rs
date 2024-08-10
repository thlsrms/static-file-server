use std::path::PathBuf;

use axum::{
    body::Body,
    http::{response::Response, HeaderValue},
    middleware::map_response,
    routing::get_service,
    Router,
};
use clap::Parser;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeDir, CompressionLevel};

#[derive(Parser)]
#[clap(name = "A simple static file server")]
#[command(version)]
struct FileServerArgs {
    #[clap(short = 'p', long = "path", default_value = "static")]
    path: PathBuf,
    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1:3000")]
    address: String,
    #[clap(short = 'z', long = "zstd-level", default_value_t = 9)]
    zstd_compression: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = FileServerArgs::parse();

    axum::serve(
        TcpListener::bind(&args.address).await.unwrap(),
        Router::new().fallback(
            get_service(ServeDir::new(&args.path).precompressed_zstd())
                .layer(
                    CompressionLayer::new()
                        .zstd(args.zstd_compression > 0)
                        .quality(CompressionLevel::Precise(args.zstd_compression)),
                )
                .layer(map_response(|mut response: Response<Body>| async {
                    response.headers_mut().insert(
                        "Cross-Origin-Opener-Policy",
                        HeaderValue::from_static("same-origin"),
                    );
                    response.headers_mut().insert(
                        "Cross-Origin-Embedder-Policy",
                        HeaderValue::from_static("require-corp"),
                    );
                    response
                })),
        ),
    )
    .await?;
    Ok(())
}
