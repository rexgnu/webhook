# Webhook Handler CLI Tool

A Rust CLI tool for local webhook testing with an interactive TUI interface. Listens on `localhost:9080`, logs all incoming HTTP requests, and displays them in real-time.

## Features

- **HTTP Server**: Catches all routes and HTTP methods
- **Real-time TUI**: Interactive terminal interface with request list and details
- **Request Details**: View method, path, headers, query params, body, and timestamps
- **Configurable Responses**: Return custom responses based on path/method
- **JSON Pretty-Print**: Automatically formats JSON bodies for readability
- **Keyboard Navigation**: Vim-style keybindings (j/k) and arrow keys

## Installation

Build the project:
```bash
cargo build --release
```

The binary will be available at `./target/release/webhook`

## Usage

Run the webhook handler:
```bash
./target/release/webhook
```

Or with cargo:
```bash
cargo run
```

The server will start listening on `localhost:9080` and display an interactive TUI.

## Configuration

Create a `config.yaml` file in the project directory or at `~/.config/webhook/config.yaml`:

```yaml
# Server settings
port: 9080
host: "127.0.0.1"

# Default response for all requests
response:
  status: 200
  headers:
    Content-Type: "application/json"
  body: '{"status": "ok"}'

# Path-specific responses
routes:
  - path: "/health"
    method: "GET"
    response:
      status: 200
      body: '{"healthy": true}'
```

## TUI Interface

The interface is divided into three sections:

### Left Pane (30%)
- Scrollable list of captured requests
- Shows timestamp, HTTP method, and path
- Newest requests at the top
- Selected request is highlighted

### Right Pane (70%)
- Detailed view of selected request:
  - Full timestamp
  - HTTP method and complete URL
  - Headers (sorted alphabetically)
  - Body (with JSON pretty-printing)

### Status Bar
- Current listening address
- Request count
- Keyboard shortcuts

## Keybindings

- `q` or `Esc` - Quit the application
- `c` - Clear all requests
- `↑` or `k` - Move selection up
- `↓` or `j` - Move selection down
- `Enter` - Expand/collapse body view
- `Page Up` - Scroll detail pane up
- `Page Down` - Scroll detail pane down

## Testing

Test the webhook handler with curl:

```bash
# POST request with JSON body
curl -X POST localhost:9080/webhook \
  -H "Content-Type: application/json" \
  -d '{"test": true, "message": "Hello"}'

# GET request
curl localhost:9080/health

# PUT request with data
curl -X PUT localhost:9080/api/data \
  -H "Authorization: Bearer token123" \
  -d "hello world"

# DELETE request
curl -X DELETE localhost:9080/api/resource/123
```

All requests will appear in real-time in the TUI.

## Project Structure

```
webhook/
├── Cargo.toml
├── config.yaml           # Configuration file
├── README.md
└── src/
    ├── main.rs           # Entry point, spawns server + TUI
    ├── server.rs         # Axum HTTP server with catch-all handler
    ├── config.rs         # Configuration loading/parsing
    ├── request.rs        # Request model and formatting
    └── ui/
        ├── mod.rs        # TUI module entry point
        ├── app.rs        # App state and event handling
        └── render.rs     # UI rendering logic
```

## Tech Stack

- **HTTP Server**: axum (async, ergonomic, tokio-based)
- **TUI Framework**: ratatui + crossterm
- **Async Runtime**: tokio
- **Serialization**: serde + serde_json + serde_yaml
- **Date/Time**: chrono

## License

MIT
