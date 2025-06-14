# Quotes Server

A simple web server that serves inspirational quotes using Rust, Axum web framework, and Askama templating engine with SQLite database storage. Features a RESTful API with OpenAPI documentation and interactive Swagger UI.

## Author

Chia-Wei Hsu (chiawei@pdx.edu)

## Features

- Get a random quote
- Browse all quotes in the database
- RESTful API for programmatic access with OpenAPI documentation
- Interactive Swagger UI for API exploration
- JWT authentication for protected endpoints
- Automatic database initialization from CSV

## Technology Stack

- **Rust** - Programming language
- **Axum** - Web framework
- **Askama** - Templating engine
- **SQLite** - Database for storing quotes and tags
- **jsonwebtoken** - JWT authentication
- **utoipa** - OpenAPI documentation generation
- **Swagger UI** - Interactive API documentation

## Dependencies

- askama
- axum
- serde
- tokio
- tower-http = { version = "0.6.2", features = ["fs", "trace"] }
- sqlx
- jsonwebtoken
- utoipa
- See `Cargo.toml` for a complete list

## Setup

1. Make sure you have Rust installed
2. Clone this repository
3. Run `cargo build --release` to compile the project
4. Create a `credentials.txt` file with your JWT secret (or set `JWT_SECRET` environment variable)
5. The application will automatically:
   - Create a `db` directory if it doesn't exist
   - Initialize a SQLite database at `db/quotes.db`
   - Run all database migrations from the `migrations` folder
   - Import default quotes from `assets/static/default_quotes.csv` if the database is empty
6. (Optional) A WASM [Quote Client](https://github.com/planetaska/quote-client) is also available for accessing the server.

## Running the Application

### Local Development

Start the server with:

```bash
cargo run --release
```

The server will be available at: `http://localhost:3000`

### Docker

Build and run using Docker:

```bash
# Build the Docker image
docker build -t quote-server .

# Run the container (uses built-in database)
docker run -p 3000:3000 quote-server

# Run with persistent database (optional)
docker run -p 3000:3000 -v $(pwd)/db:/app/db quote-server
```

The container includes a pre-initialized database with all migrations and default quotes. If you mount a volume to `/app/db`, the container will use your existing database if present, or copy the built-in database to the mounted location if not found.

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
- `POST /api/v1/quotes` - Create a new quote (requires JWT authentication)
- `PUT /api/v1/quotes/{id}` - Update an existing quote (requires JWT authentication)
- `DELETE /api/v1/quotes/{id}` - Delete a quote by ID (requires JWT authentication)
- `POST /auth` - Register and get JWT token

### Documentation
- `GET /swagger-ui` - Interactive Swagger UI for API exploration
- `GET /api-docs/openapi.json` - OpenAPI specification in JSON format

## API Documentation

You can explore the API interactively using the Swagger UI at:

```
http://localhost:3000/swagger-ui
```

### Authentication

Protected endpoints require JWT authentication. Register with your credentials to get a token:

```bash
curl -X POST http://localhost:3000/auth \
  -H "Content-Type: application/json" \
  -d '{"full_name": "Your Name", "email": "your@email.com", "password": "your_secret"}'
```

Use the returned token in the Authorization header:
```bash
curl -H "Authorization: Bearer <token>" http://localhost:3000/api/v1/quotes
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
│   ├── authjwt.rs              # JWT authentication module
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

## License

MIT license