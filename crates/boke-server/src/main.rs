use axum::{
    routing::{delete, get, post, put},
    Router,
};
use boke_core::{
    db::DatabasePool,
    services::{ArticleService, FeedService, FolderService},
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod routes;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabasePool,
    pub feed_service: Arc<FeedService<DatabasePool>>,
    pub article_service: Arc<ArticleService<DatabasePool>>,
    pub folder_service: Arc<FolderService<DatabasePool>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "boke_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting Boke server with config: {:?}", config);

    // Initialize database
    let db = DatabasePool::from_url(&config.database_url).await?;
    tracing::info!("Database connected");

    // Initialize services
    let db_arc = Arc::new(db.clone());
    let state = AppState {
        db: db.clone(),
        feed_service: Arc::new(FeedService::new(db_arc.clone())),
        article_service: Arc::new(ArticleService::new(db_arc.clone())),
        folder_service: Arc::new(FolderService::new(db_arc)),
    };

    // Build router
    let api_routes = Router::new()
        // Feed routes
        .route("/feeds", get(routes::feeds::get_feeds))
        .route("/feeds", post(routes::feeds::add_feed))
        .route("/feeds/{id}", delete(routes::feeds::remove_feed))
        .route("/feeds/{id}/refresh", post(routes::feeds::refresh_feed))
        .route("/feeds/refresh", post(routes::feeds::refresh_all_feeds))
        // Article routes
        .route("/articles", get(routes::articles::get_articles))
        .route("/articles/{id}", get(routes::articles::get_article))
        .route("/articles/{id}/read", post(routes::articles::toggle_read))
        .route(
            "/articles/{id}/favorite",
            post(routes::articles::toggle_favorite),
        )
        .route(
            "/articles/{id}/content",
            get(routes::articles::fetch_article_content),
        )
        .route("/articles/mark-read", post(routes::articles::mark_all_read))
        .route(
            "/articles/mark-unread",
            post(routes::articles::mark_all_unread),
        )
        .route("/articles/search", get(routes::articles::search_articles))
        .route(
            "/articles/favorites/count",
            get(routes::articles::get_favorites_count),
        )
        // Folder routes
        .route("/folders", get(routes::folders::get_folders))
        .route("/folders", post(routes::folders::create_folder))
        .route("/folders/{id}", put(routes::folders::rename_folder))
        .route("/folders/{id}", delete(routes::folders::delete_folder))
        .route(
            "/folders/{id}/feeds/{feed_id}",
            put(routes::folders::move_feed_to_folder),
        );

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(
            ServeDir::new(&config.static_dir).append_index_html_on_directories(true),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr: SocketAddr = config.bind_address.parse()?;
    tracing::info!("Boke server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
