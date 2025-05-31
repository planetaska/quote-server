# Quotes Server

A simple web server that serves inspirational quotes using Rust, Axum web framework, and Askama templating engine with SQLite database storage. Features a RESTful API with OpenAPI documentation and interactive Swagger UI.

## Features

- Get a random quote
- Browse all quotes in the database
- RESTful API for programmatic access with OpenAPI documentation
- Interactive Swagger UI for API exploration
- Automatic database initialization from CSV

## Technology Stack

- **Rust** - Programming language
- **Axum** - Web framework
- **Askama** - Templating engine
- **SQLite** - Database for storing quotes and tags
- **utoipa** - OpenAPI documentation generation
- **Swagger UI** - Interactive API documentation

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
- utoipa (with chrono feature) - OpenAPI documentation
- utoipa-axum - Axum integration for utoipa
- utoipa-swagger-ui - Swagger UI integration

## Setup

1. Make sure you have Rust installed
2. Clone this repository
3. Run `cargo build --release` to compile the project
4. The application will automatically:
   - Create a `db` directory if it doesn't exist
   - Initialize a SQLite database at `db/quotes.db`
   - Run all database migrations from the `migrations` folder
   - Import default quotes from `assets/static/default_quotes.csv` if the database is empty

## Running the Application

Start the server with:

```bash
cargo run --release
```

The server will be available at: `http://localhost:3000`

## Available Endpoints

### Web Interface
- `GET /` - Home page with a random quote
- `GET /about` - About page with technical details
- `GET /quotes` - View all quotes
- `GET /quote/random` - View a random quote

### API Endpoints
- `GET /api/v1/quotes` - Get all quotes as JSON
- `GET /api/v1/quotes/{id}` - Get a specific quote by ID as JSON
- `GET /api/v1/quotes/random` - Get a random quote as JSON
- `POST /api/v1/quotes` - Create a new quote
- `PUT /api/v1/quotes/{id}` - Update an existing quote
- `DELETE /api/v1/quotes/{id}` - Delete a quote by ID

### Documentation
- `GET /swagger-ui` - Interactive Swagger UI for API exploration
- `GET /api-docs/openapi.json` - OpenAPI specification in JSON format

## API Documentation

You can explore the API interactively using the Swagger UI at:

```
http://localhost:3000/swagger-ui
```

### Example API Response

```json
{
  "id": 1,
  "quote": "The only thing we have to fear is fear itself.",
  "source": "Franklin D. Roosevelt",
  "created_at": "2024-01-01T12:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z",
  "tags": ["fear", "courage", "inspiration"]
}
```

## Database Structure

The application uses SQLite with two main tables:

```sql
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

*Since SQLite's INTEGER type already represents a 64-bit integer, and it doesn't distinguish a separate BIGINT type, using INTEGER here is sufficient.

## Project Structure

```
.
├── assets/
│   ├── static/
│   │   └── default_quotes.csv  # Default quotes for database initialization
│   └── templates/
│       ├── about.html          # About page template
│       ├── index.html          # Home page template
│       ├── layout.html         # Base layout template
│       ├── nav.html            # Navigation component
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
│   ├── api.rs                  # API endpoints with OpenAPI documentation
│   ├── db.rs                   # Database interaction code
│   ├── main.rs                 # Application entry point and routing
│   └── templates.rs            # Template handling code
├── askama.toml                 # Askama configuration
├── Cargo.toml                  # Cargo package configuration
└── README.md                   # This file
```

## Development

To run tests:

```bash
cargo test
```

To build for release:

```bash
cargo build --release
```

If you changed database schema, make sure to set DATABASE_URL env and re-run the sqlx generation:

```bash
export DATABASE_URL=sqlite://db/quotes.db
cargo sqlx prepare
```

## Author

Chia-Wei Hsu (chiawei@pdx.edu)