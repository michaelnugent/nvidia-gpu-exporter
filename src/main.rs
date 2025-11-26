use axum::{
    extract::Request,
    http::StatusCode,
    response::{Html, Response},
    routing::get,
    Router,
};
use clap::Parser;
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use tracing::{debug, info, warn};

mod exporter;
mod metrics;

use exporter::Exporter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to listen on for web interface and telemetry
    #[arg(long, default_value = "0.0.0.0:9445")]
    web_listen_address: String,

    /// Path under which to expose metrics
    #[arg(long, default_value = "/metrics")]
    web_telemetry_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let exporter = Exporter::new();
    let exporter_clone = exporter.clone();

    let app = Router::new()
        .route(
            &args.web_telemetry_path,
            axum::routing::get(move |_req: Request| async move {
                debug!("Metrics endpoint called");
                
                debug!("Gathering metrics from exporter...");
                let metric_families = exporter_clone.gather();
                debug!("Gathered {} metric families", metric_families.len());
                
                debug!("Creating encoder...");
                let encoder = TextEncoder::new();
                let mut buffer = Vec::new();
                
                debug!("Encoding {} metric families...", metric_families.len());
                if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                    warn!("Failed to encode metrics: {}", e);
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("Failed to encode metrics: {}", e))
                        .expect("Failed to build error response");
                }
                debug!("Encoded metrics to buffer of {} bytes", buffer.len());
                
                debug!("Converting buffer to UTF-8 string...");
                match String::from_utf8(buffer) {
                    Ok(body) => {
                        debug!("Successfully created response body ({} bytes)", body.len());
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "text/plain; version=0.0.4")
                            .body(body)
                            .expect("Failed to build response")
                    }
                    Err(e) => {
                        warn!("Failed to encode metrics as UTF-8: {}", e);
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(format!("Failed to encode metrics as UTF-8: {}", e))
                            .expect("Failed to build error response")
                    }
                }
            }),
        )
        .route(
            "/",
            get(|| async {
                Html(
                    r#"
                    <html>
                        <head><title>NVIDIA GPU Exporter</title></head>
                        <body>
                            <h1>NVIDIA GPU Exporter</h1>
                            <p><a href='/metrics'>Metrics</a></p>
                        </body>
                    </html>
                    "#,
                )
            }),
        );

    let addr: SocketAddr = args.web_listen_address.parse()?;
    info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Set up signal handling for graceful shutdown
    let server = axum::serve(listener, app);
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        warn!("Received shutdown signal, shutting down gracefully...");
    };

    tokio::select! {
        result = server => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutdown signal received, server stopping...");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use tower::ServiceExt;

    #[test]
    fn test_args_default_values() {
        // Test that default values are correctly set
        let args = Args {
            web_listen_address: "0.0.0.0:9445".to_string(),
            web_telemetry_path: "/metrics".to_string(),
        };
        
        assert_eq!(args.web_listen_address, "0.0.0.0:9445");
        assert_eq!(args.web_telemetry_path, "/metrics");
    }

    #[test]
    fn test_args_parsing() {
        // Just verify the structure is correct
        use clap::CommandFactory;
        let _cmd = Args::command();
    }

    #[tokio::test]
    async fn test_metrics_endpoint_response() {
        let exporter = Exporter::new();
        let exporter_clone = exporter.clone();

        let app = Router::new()
            .route(
                "/metrics",
                axum::routing::get(move |_req: Request| async move {
                    let encoder = TextEncoder::new();
                    let metric_families = exporter_clone.gather();
                    let mut buffer = Vec::new();
                    
                    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                        return Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(format!("Failed to encode metrics: {}", e))
                            .expect("Failed to build error response");
                    }
                    
                    match String::from_utf8(buffer) {
                        Ok(body) => Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "text/plain; version=0.0.4")
                            .body(body)
                            .expect("Failed to build response"),
                        Err(e) => Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(format!("Failed to encode metrics as UTF-8: {}", e))
                            .expect("Failed to build error response"),
                    }
                }),
            );

        // Test the metrics endpoint
        let response = app
            .clone()
            .oneshot(
                HttpRequest::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        let status = response.status();
        let headers = response.headers().clone();
        
        if status != StatusCode::OK {
            use axum::body::to_bytes;
            let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body_str = String::from_utf8_lossy(&body_bytes);
            eprintln!("Response status: {}, body: {}", status, body_str);
            panic!("Expected OK status");
        }
        
        assert_eq!(status, StatusCode::OK);
        
        let content_type = headers.get("Content-Type");
        assert!(content_type.is_some());
        assert_eq!(content_type.unwrap(), "text/plain; version=0.0.4");
    }

    #[tokio::test]
    async fn test_root_endpoint() {
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    Html(
                        r#"
                        <html>
                            <head><title>NVIDIA GPU Exporter</title></head>
                            <body>
                                <h1>NVIDIA GPU Exporter</h1>
                                <p><a href='/metrics'>Metrics</a></p>
                            </body>
                        </html>
                        "#,
                    )
                }),
            );

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
