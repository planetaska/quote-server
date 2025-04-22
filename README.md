# Quotes Server

A simple web server that serves quotes using Axum web framework and Askama templating engine.

## Dependencies

- askama = "0.13.1"
- axum = "0.8.3"
- serde = { version = "1.0.219", features = ["derive"] }
- serde_json = "1.0.140"
- tokio = { version = "1.44.2", features = ["full"] }
- tower-http = { version = "0.6.2", features = ["fs", "trace"] }
- tracing = "0.1.41"

## Setup

1. Make sure you have Rust installed
2. Clone this repository
3. Run `cargo build`
4. Start the server with `cargo run`

## Features

- Serves quotes through HTTP endpoints
- Uses Askama templates for HTML rendering
- RESTful API design
- Async request handling with Axum

## Usage

Once running, access the server at `http://localhost:3000`

## Development

The project uses:

- Axum for handling HTTP requests
- Askama for template rendering
- Tokio for async runtime

## Author

Chia-Wei Hsu (chiawei@pdx.edu)
