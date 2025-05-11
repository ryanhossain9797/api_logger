use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use limbo::Builder;
use serde::{Deserialize, Serialize};
use std::fs;
use tower_http::cors::CorsLayer;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    message: String,
    timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SqlQuery {
    query: String,
}

#[axum::debug_handler]
async fn add_log(
    State(db): State<limbo::Connection>,
    Json(payload): Json<LogEntry>,
) -> Result<StatusCode, StatusCode> {
    let timestamp = payload
        .timestamp
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

    db.execute(
        "INSERT INTO logs (message, timestamp) VALUES (?, ?)",
        (&payload.message as &str, &timestamp as &str),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

#[axum::debug_handler]
async fn execute_query(
    State(db): State<limbo::Connection>,
    Json(payload): Json<SqlQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if the query is read-only (starts with SELECT)
    if !payload.query.trim().to_uppercase().starts_with("SELECT") {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut result = db
        .query(&payload.query, ())
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut rows = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(row)) => {
                let mut map = serde_json::Map::new();
                let value = match row.get_value(0) {
                    Ok(v) => match v {
                        limbo::Value::Null => serde_json::Value::Null,
                        limbo::Value::Integer(i) => serde_json::Value::Number(i.into()),
                        limbo::Value::Real(f) => serde_json::Value::Number(
                            serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
                        ),
                        limbo::Value::Text(s) => serde_json::Value::String(s),
                        limbo::Value::Blob(_) => serde_json::Value::Null,
                    },
                    Err(_) => serde_json::Value::Null,
                };
                map.insert("value".to_string(), value);
                rows.push(serde_json::Value::Object(map));
            }
            Ok(None) => break,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    }

    println!("rows: {:?}", rows);

    Ok(Json(serde_json::Value::Array(rows)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read config
    let config_str = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_str)?;

    // Initialize database
    let db = Builder::new_local("log.db").build().await?;
    let conn = db.connect()?;

    // Create logs table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        (),
    )
    .await?;

    // Build router
    let app = Router::new()
        .route("/log", post(add_log))
        .route("/query", post(execute_query))
        .layer(CorsLayer::permissive())
        .with_state(conn);

    // Start server
    let addr = format!("localhost:{}", config.port);
    println!("Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
