use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    routing::post,
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
    key: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SqlQuery {
    key: Option<String>,
    value_like: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogRow {
    id: i64,
    key: String,
    value: String,
    timestamp: String,
}

#[axum::debug_handler]
async fn add_log(
    State(db): State<limbo::Connection>,
    Json(payload): Json<LogEntry>,
) -> Result<StatusCode, StatusCode> {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!(
        "Inserting log - Key: {}, Value: {}",
        payload.key, payload.value
    );

    db.execute(
        "INSERT INTO logs (key, value, timestamp) VALUES (?, ?, ?)",
        (
            &payload.key as &str,
            &payload.value as &str,
            &timestamp as &str,
        ),
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
    let mut query = String::from("SELECT id, key, value, timestamp FROM logs WHERE 1=1");
    let mut params: Vec<&str> = Vec::new();

    if let Some(key) = &payload.key {
        query.push_str(" AND key = ?");
        params.push(key);
    }

    if let Some(value_like) = &payload.value_like {
        query.push_str(" AND value LIKE ?");
        params.push(value_like);
    }

    if let Some(from) = &payload.from {
        query.push_str(" AND timestamp > ?");
        params.push(from);
    }

    if let Some(to) = &payload.to {
        query.push_str(" AND timestamp < ?");
        params.push(to);
    }

    let mut result = db
        .query(&query, params)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut rows = Vec::new();
    let mut count = 0;
    loop {
        match result.next().await {
            Ok(Some(row)) => {
                let log_row = LogRow {
                    id: match row.get_value(0) {
                        Ok(limbo::Value::Integer(i)) => i,
                        _ => 0,
                    },
                    key: match row.get_value(1) {
                        Ok(limbo::Value::Text(s)) => s,
                        _ => "".to_string(),
                    },
                    value: match row.get_value(2) {
                        Ok(limbo::Value::Text(s)) => s,
                        _ => "".to_string(),
                    },
                    timestamp: match row.get_value(3) {
                        Ok(limbo::Value::Text(s)) => s,
                        _ => "".to_string(),
                    },
                };
                if count < 3 {
                    println!(
                        "Row {}: Key: {}, Value: {}, Timestamp: {}",
                        count + 1,
                        log_row.key,
                        log_row.value,
                        log_row.timestamp
                    );
                }
                rows.push(serde_json::to_value(log_row).unwrap_or(serde_json::Value::Null));
                count += 1;
            }
            Ok(None) => break,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    }

    println!("Total rows returned: {}", count);
    Ok(Json(serde_json::Value::Array(rows)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read or create config
    let config = if let Ok(config_str) = fs::read_to_string("config.json") {
        serde_json::from_str(&config_str)?
    } else {
        println!("config.json not found, creating default with port 80");
        let config = Config { port: 80 };
        fs::write("config.json", serde_json::to_string_pretty(&config)?)?;
        config
    };

    // Initialize database
    let db = Builder::new_local("log.db").build().await?;
    let conn = db.connect()?;

    // Create logs table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
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
