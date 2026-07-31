mod api;
mod error;
mod openapi;
mod static_files;

use std::{net::IpAddr, path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use appstore_api::ClientConfig;
use asapi_app::{AppService, ProjectManager};
use axum::Router;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Debug, Clone)]
pub struct ServeConfig {
    pub host: IpAddr,
    pub port: u16,
    /// Optional base directory. Storage is placed below
    /// `<base>/asapi-storage/projects`.
    pub storage_base: Option<PathBuf>,
    pub client: ClientConfig,
}

pub async fn serve(config: ServeConfig) -> Result<()> {
    init_tracing();
    let manager = ProjectManager::open(config.storage_base).await?;
    let service = AppService::new(manager.clone(), config.client)?;
    let app = router(service.clone());
    let address = (config.host, config.port);
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind {}:{}", config.host, config.port))?;
    let local_address = listener.local_addr()?;

    tracing::info!("asapi is available at http://{local_address}");
    println!("asapi app: http://{local_address}");
    println!("API reference: http://{local_address}/api/openapi.json");

    let refresh_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15 * 60));
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(error) = service.refresh_stale().await {
                tracing::warn!(%error, "automatic refresh pass failed");
            }
        }
    });

    let server_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server stopped unexpectedly");

    refresh_task.abort();
    let _ = refresh_task.await;
    manager
        .checkpoint_and_close()
        .await
        .context("failed to checkpoint project databases during shutdown")?;

    server_result
}

pub fn router(service: AppService) -> Router {
    api::router(service)
        .fallback(static_files::serve)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "asapi_server=info,tower_http=info".into()),
        )
        .try_init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to listen for Ctrl-C");
        }
    };

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let terminate = async {
            match signal(SignalKind::terminate()) {
                Ok(mut signal) => {
                    signal.recv().await;
                }
                Err(error) => tracing::error!(%error, "failed to listen for SIGTERM"),
            }
        };

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }
    }

    #[cfg(not(unix))]
    ctrl_c.await;
}
