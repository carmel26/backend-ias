use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod error;
mod handlers;
mod models;

use handlers::{
    admin_handler::*, analysis_handler::*, assessment_handler::*, auth_handler::*,
    dashboard_handler::*, question_handler::*, report_handler::*, user_handler::*,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing logger
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Academic Assessment Item Analysis Backend Server...");

    // Initialize SurrealDB database & seed initial data
    let db = db::init_db().await?;

    // Enable CORS for frontend development & production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build Axum Router
    let app = Router::new()
        // Auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(get_me))

        // User & Admin management routes
        .route("/api/admin/users", get(list_users).post(create_admin_user))
        .route("/api/admin/users/:id/role", put(update_user_role))
        .route("/api/admin/users/:id", delete(delete_user))

        // System Settings routes
        .route("/api/admin/settings", get(get_settings).put(update_settings))

        // Dashboard stats
        .route("/api/dashboard/stats", get(get_dashboard_stats))

        // Assessment routes
        .route(
            "/api/assessments",
            get(list_assessments).post(create_assessment),
        )
        .route(
            "/api/assessments/:id",
            get(get_assessment)
                .put(update_assessment)
                .delete(delete_assessment),
        )
        .route(
            "/api/assessments/:id/submit",
            post(submit_assessment),
        )
        .route(
            "/api/assessments/:id/verify",
            post(verify_assessment),
        )

        // Questions / Item Entry routes
        .route(
            "/api/assessments/:id/questions",
            get(list_questions).post(add_question),
        )
        .route(
            "/api/assessments/:id/questions/batch",
            put(batch_save_questions),
        )
        .route("/api/questions/:id", delete(delete_question))

        // Analysis & Report routes
        .route(
            "/api/assessments/:id/analyze",
            get(analyze_assessment).post(analyze_assessment),
        )
        .route(
            "/api/assessments/:id/report",
            get(get_assessment_report),
        )

        // Apply CORS
        .layer(cors)

        // Attach database state
        .with_state(db);

    // ---------------------------------------------------------
    // Server binding
    // ---------------------------------------------------------
    //
    // Render provides the PORT environment variable.
    // Locally, if PORT is not set, port 3000 is used.
    //
    // IMPORTANT:
    // 0.0.0.0 allows Render to access the application.
    // ---------------------------------------------------------

    let host = std::env::var("SERVER_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string());

    let addr = format!("{}:{}", host, port);

    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
