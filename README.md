# Quotes Server

A simple web server that serves inspirational quotes using Rust, Axum web framework, and Askama templating engine with SQLite database storage.

## Features

- Get a random quote
- Browse all quotes in the database
- RESTful API for programmatic access
- Automatic database initialization from CSV

## Technology Stack

- **Rust** - Programming language
- **Axum** - Web framework
- **Askama** - Templating engine
- **SQLite** - Database for storing quotes and tags

## Dependencies

- askama = "0.14.0"
- axum = "0.8.3"
- fastrand = "2.3.0"
- serde = { version = "1.0.219", features = ["derive"] }
- serde_json = "1.0.140"
- tokio = { version = "1.44.2", features = ["full"] }
- tower-http = { version = "0.6.2", features = ["fs", "trace"] }
- tracing = "0.1.41"
- sqlx = { version = "0.8.5", features = ["runtime-tokio", "sqlite", "derive", "macros", "migrate", "chrono", "json"] }
- csv = "1.3.0"

## Setup

1. Make sure you have Rust installed
2. Clone this repository
3. Run `cargo build` to compile the project
4. The application will automatically:
    - Create a `db` directory if it doesn't exist
    - Initialize a SQLite database at `db/quotes.db`
    - Run all database migrations from the `migrations` folder
    - Import default quotes from `assets/static/default_quotes.csv` if the database is empty

## Running the Application

Start the server with:

```
cargo run
```

The server will be available at: `http://localhost:3000`

## Available Endpoints

- `GET /` - Home page
- `GET /about` - About page with technical details
- `GET /quotes` - View all quotes
- `GET /quote/random` - View a random quote
- `GET /api/quotes` - API endpoint to get all quotes as JSON
- `GET /api/quotes/random` - API endpoint to get a random quote as JSON

## Database Structure

The application uses SQLite with two main tables:

```
quotes
    - id: Integer (Primary Key)
    - quote: Text
    - source: Text
    - created_at: DateTime
    - updated_at: DateTime

tags
    - id: Integer (Primary Key)
    - quote_id: Integer (Foreign Key)
    - name: Text
    - created_at: DateTime
    - updated_at: DateTime
```

\*Since SQLite’s INTEGER type already represents a 64-bit integer, and it doesn’t distinguish a separate BIGINT type, using INTEGER here is sufficient.

## Project Structure

```
.
├── assets/
│   ├── static/
│   │   └── default_quotes.csv  # Default quotes for database initialization
│   └── templates/
│       ├── about.html          # About page template
│       ├── index.html          # Home page template
│       ├── quote.html          # Single quote template
│       └── quotes.html         # All quotes template
├── db/
│   └── quotes.db               # SQLite database (created automatically)
├── migrations/                 # Database migration files
│   ├── 20250425230811_create_quotes.up.sql
│   ├── 20250425230811_create_quotes.down.sql
│   ├── 20250425231048_create_tags.up.sql
│   └── 20250425231048_create_tags.down.sql
├── src/
│   ├── db.rs                   # Database interaction code
│   ├── main.rs                 # Application entry point
│   └── templates.rs            # Template handling code
├── Cargo.toml                  # Cargo package configuration
└── README.md                   # This file
```

## Development

To run tests:

You will need to set DATABASE_URL env variable to run the database tests.

```
export DATABASE_URL="sqlite://db/quotes.db"

cargo test
```

## Author

Chia-Wei Hsu (chiawei@pdx.edu)