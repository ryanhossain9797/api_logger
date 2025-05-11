# API Logger

A simple and efficient API logging service built with Rust, Limbo and Axum. This service provides endpoints to log key-value pairs and query the logs with flexible filtering options.

## Features

- Simple key-value logging
- SQL-like querying capabilities
- CORS support
- Limbo SQLite like database backend
- Configurable port

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

#### Windows-specific Requirements
- Visual C++ Redistributable for Visual Studio 2022:
  - [x64 version](https://aka.ms/vs/17/release/vc_redist.x64.exe)
  - [x86 version](https://aka.ms/vs/17/release/vc_redist.x86.exe)

### Building from Source

1. Clone the repository:
```bash
git clone <repository-url>
cd api_logger
```

2. Build the project:
```bash
cargo build --release
```

The compiled binary will be available at `target/release/api_logger`.

### Cross-Platform Building

To build for different platforms, you can use cargo's cross-compilation features:

1. Install cross:
```bash
cargo install cross
```

2. Build for specific targets:
```bash
# For Windows
cross build --release --target x86_64-pc-windows-msvc

# For Linux
cross build --release --target x86_64-unknown-linux-gnu

# For macOS
cross build --release --target x86_64-apple-darwin
```

## Configuration

The service uses a `config.json` file for configuration. If the file doesn't exist, it will be created with default settings.

Example `config.json`:
```json
{
    "port": 80
}
```

### Configuration Options

- `port`: The port number on which the server will listen (default: 80)

## API Endpoints

### 1. Add Log Entry

**Endpoint:** `POST /log`

**Request Body:**
```json
{
    "key": "string",
    "value": "string"
}
```

**Response:**
- Status: 201 Created (on success)
- Status: 500 Internal Server Error (on failure)

### 2. Query Logs

**Endpoint:** `POST /query`

**Request Body:**
```json
{
    "key": "string",           // Optional: Filter by exact key match
    "value_like": "string",    // Optional: Filter by value using SQL LIKE
    "from": "string",          // Optional: Filter by timestamp (greater than)
    "to": "string"             // Optional: Filter by timestamp (less than)
}
```

**Response:**
```json
[
    {
        "id": 1,
        "key": "string",
        "value": "string",
        "timestamp": "YYYY-MM-DD HH:MM:SS"
    }
]
```

## Database

The service uses Limbo as its database backend, which provides a SQL-like interface. The database file (`log.db`) will be created automatically in the same directory as the executable.

### Schema

```sql
CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    timestamp TEXT NOT NULL
)
```

## Running the Service

1. Start the service:
```bash
./target/release/api_logger
```

2. The service will start listening on the configured port (default: 80)

## Error Handling

- Invalid JSON payloads will result in 400 Bad Request
- Database errors will result in 500 Internal Server Error
- Query errors will result in 400 Bad Request

## CORS

The service has permissive CORS settings enabled, allowing requests from any origin. This can be modified in the code if more restrictive CORS policies are needed.

