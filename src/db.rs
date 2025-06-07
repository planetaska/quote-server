//! Database interaction module for the Quotes Server.
//!
//! Provides functions for SQLite database initialization, migration handling,
//! importing default quotes from CSV, and CRUD operations for quotes and tags.
//!
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::{collections::HashSet, fs, path::Path};
use tracing::info;
use utoipa::ToSchema;

const DB_URL: &str = "sqlite://db/quotes.db";

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteFromCsv {
    pub id: i64,
    pub quote: String,
    pub source: String,
    pub tags: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub id: i64,
    pub quote: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub quote_id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuoteWithTags {
    /// Unique identifier for the quote
    #[schema(example = 1)]
    pub id: i64,
    /// The actual quote text
    #[schema(example = "The only way to do great work is to love what you do.")]
    pub quote: String,
    /// Source or author of the quote
    #[schema(example = "Steve Jobs")]
    pub source: String,
    /// Timestamp when the quote was created
    #[schema(value_type = String, format = DateTime, example = "2024-01-01T12:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Timestamp when the quote was last updated
    #[schema(value_type = String, format = DateTime, example = "2024-01-01T12:00:00Z")]
    pub updated_at: DateTime<Utc>,
    /// List of tags associated with the quote
    #[schema(example = json!(["motivation", "work", "success"]))]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateQuoteRequest {
    /// The quote text
    #[schema(example = "The future belongs to those who believe in the beauty of their dreams.")]
    pub quote: String,
    /// Source or author of the quote
    #[schema(example = "Eleanor Roosevelt")]
    pub source: String,
    /// Optional list of tags for the quote
    #[schema(example = json!(["dreams", "future", "motivation"]))]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateQuoteRequest {
    /// The updated quote text
    #[schema(example = "The future belongs to those who believe in the beauty of their dreams.")]
    pub quote: String,
    /// Updated source or author of the quote
    #[schema(example = "Eleanor Roosevelt")]
    pub source: String,
    /// Updated list of tags for the quote (replaces existing tags)
    #[schema(example = json!(["dreams", "future", "motivation", "inspiration"]))]
    pub tags: Option<Vec<String>>,
}

pub async fn init_db() -> Result<Pool<Sqlite>, sqlx::Error> {
    // Create db directory if it doesn't exist
    let db_dir = Path::new("db");
    if !db_dir.exists() {
        fs::create_dir_all(db_dir).expect("Failed to create db directory");
    }

    // Check if database exists, if not create it
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Database does not exist. Creating...");
        Sqlite::create_database(DB_URL).await?;
    }

    // Connect to SQLite database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await?;

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Check if quotes table is empty, if so populate from CSV
    let count = sqlx::query!("SELECT COUNT(*) as count FROM quotes")
        .fetch_one(&pool)
        .await?;

    if count.count == 0 {
        info!("Quotes table is empty. Importing from CSV...");
        import_quotes_from_csv(&pool).await?;
    }

    Ok(pool)
}

async fn import_quotes_from_csv(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Read CSV file
    let csv_path = "assets/static/default_quotes.csv";
    let csv_content = fs::read_to_string(csv_path).expect("Failed to read CSV file");

    // Parse CSV
    let mut rdr = csv::Reader::from_reader(csv_content.as_bytes());
    let mut quotes: Vec<QuoteFromCsv> = Vec::new();

    for result in rdr.deserialize() {
        let record: QuoteFromCsv = result.expect("Failed to parse CSV record");
        quotes.push(record);
    }

    // Process each quote individually
    for quote in quotes {
        let now = Utc::now();

        // Insert quote
        let quote_id = sqlx::query!(
            "INSERT INTO quotes (quote, source, created_at, updated_at) VALUES (?, ?, ?, ?)",
            quote.quote,
            quote.source,
            now,
            now
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        // Process tags
        if !quote.tags.is_empty() {
            // Split tags by comma and trim spaces
            let tags: HashSet<String> = quote
                .tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Insert tags
            for tag in tags {
                // Avoid SQL injection by using parameterized query
                let _ = sqlx::query!(
                    "INSERT INTO tags (quote_id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
                    quote_id,
                    tag,
                    now,
                    now
                )
                .execute(pool)
                .await?;
            }
        }
    }

    info!("Successfully imported quotes from CSV.");
    Ok(())
}

// Function to create a new quote
pub async fn create_quote(
    pool: &Pool<Sqlite>,
    request: CreateQuoteRequest,
) -> Result<QuoteWithTags, sqlx::Error> {
    let now = Utc::now();

    // Insert the quote
    let quote_id = sqlx::query!(
        "INSERT INTO quotes (quote, source, created_at, updated_at) VALUES (?, ?, ?, ?)",
        request.quote,
        request.source,
        now,
        now
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    // Insert tags if provided
    let mut tag_names = Vec::new();
    if let Some(tags) = request.tags {
        // Use HashSet to remove duplicates and filter empty strings
        let unique_tags: HashSet<String> = tags
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for tag in &unique_tags {
            sqlx::query!(
                "INSERT INTO tags (quote_id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
                quote_id,
                tag,
                now,
                now
            )
            .execute(pool)
            .await?;
        }

        tag_names = unique_tags.into_iter().collect();
        tag_names.sort(); // Sort for consistent ordering
    }

    // Return the created quote with tags
    Ok(QuoteWithTags {
        id: quote_id,
        quote: request.quote,
        source: request.source,
        created_at: now,
        updated_at: now,
        tags: tag_names,
    })
}

// Function to update an existing quote
pub async fn update_quote(
    pool: &Pool<Sqlite>,
    quote_id: i64,
    request: UpdateQuoteRequest,
) -> Result<Option<QuoteWithTags>, sqlx::Error> {
    let now = Utc::now();

    // First, check if the quote exists and get its creation timestamp
    let existing_quote = sqlx::query!(
        "SELECT created_at as \"created_at: DateTime<Utc>\" FROM quotes WHERE id = ?",
        quote_id
    )
    .fetch_optional(pool)
    .await?;

    if existing_quote.is_none() {
        return Ok(None); // Quote doesn't exist
    }

    let created_at = existing_quote.unwrap().created_at;

    // Update the quote
    sqlx::query!(
        "UPDATE quotes SET quote = ?, source = ?, updated_at = ? WHERE id = ?",
        request.quote,
        request.source,
        now,
        quote_id
    )
    .execute(pool)
    .await?;

    // Delete existing tags for this quote
    sqlx::query!("DELETE FROM tags WHERE quote_id = ?", quote_id)
        .execute(pool)
        .await?;

    // Insert new tags if provided
    let mut tag_names = Vec::new();
    if let Some(tags) = request.tags {
        // Use HashSet to remove duplicates and filter empty strings
        let unique_tags: HashSet<String> = tags
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for tag in &unique_tags {
            sqlx::query!(
                "INSERT INTO tags (quote_id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
                quote_id,
                tag,
                now,
                now
            )
            .execute(pool)
            .await?;
        }

        tag_names = unique_tags.into_iter().collect();
        tag_names.sort(); // Sort for consistent ordering
    }

    // Return the updated quote with tags
    Ok(Some(QuoteWithTags {
        id: quote_id,
        quote: request.quote,
        source: request.source,
        created_at,
        updated_at: now,
        tags: tag_names,
    }))
}

// Function to delete a quote by ID
pub async fn delete_quote(pool: &Pool<Sqlite>, quote_id: i64) -> Result<bool, sqlx::Error> {
    // First check if the quote exists
    let exists = sqlx::query!("SELECT id FROM quotes WHERE id = ?", quote_id)
        .fetch_optional(pool)
        .await?;

    if exists.is_none() {
        return Ok(false); // Quote doesn't exist
    }

    // Delete the quote (tags will be deleted automatically due to CASCADE)
    let result = sqlx::query!("DELETE FROM quotes WHERE id = ?", quote_id)
        .execute(pool)
        .await?;

    // Return true if a row was affected (deleted)
    Ok(result.rows_affected() > 0)
}

// Function to get a quote by ID
pub async fn get_quote_by_id(
    pool: &Pool<Sqlite>,
    quote_id: i64,
) -> Result<Option<QuoteWithTags>, sqlx::Error> {
    // Query the specific quote
    let quote = sqlx::query_as!(
        Quote,
        "SELECT id, quote, source, created_at as \"created_at: DateTime<Utc>\", updated_at as \"updated_at: DateTime<Utc>\" FROM quotes WHERE id = ?",
        quote_id
    )
        .fetch_optional(pool)
        .await?;

    match quote {
        Some(quote) => {
            // Get tags for this quote
            let tags = sqlx::query_as!(
                Tag,
                "SELECT id, quote_id, name, created_at as \"created_at: DateTime<Utc>\", updated_at as \"updated_at: DateTime<Utc>\" FROM tags WHERE quote_id = ?",
                quote.id
            )
                .fetch_all(pool)
                .await?;

            // Extract tag names
            let tag_names = tags.into_iter().map(|t| t.name).collect();

            Ok(Some(QuoteWithTags {
                id: quote.id,
                quote: quote.quote,
                source: quote.source,
                created_at: quote.created_at,
                updated_at: quote.updated_at,
                tags: tag_names,
            }))
        }
        None => Ok(None),
    }
}

// Function to get all quotes with their tags
pub async fn get_all_quotes(pool: &Pool<Sqlite>) -> Result<Vec<QuoteWithTags>, sqlx::Error> {
    // Query all quotes
    let quotes = sqlx::query_as!(
        Quote,
        "SELECT id, quote, source, created_at as \"created_at: DateTime<Utc>\", updated_at as \"updated_at: DateTime<Utc>\" FROM quotes ORDER BY created_at DESC"
    )
        .fetch_all(pool)
        .await?;

    let mut quotes_with_tags = Vec::new();

    // For each quote, get its tags
    for quote in quotes {
        let tags = sqlx::query_as!(
            Tag,
            "SELECT id, quote_id, name, created_at as \"created_at: DateTime<Utc>\", updated_at as \"updated_at: DateTime<Utc>\" FROM tags WHERE quote_id = ?",
            quote.id
        )
            .fetch_all(pool)
            .await?;

        // Extract tag names
        let tag_names = tags.into_iter().map(|t| t.name).collect();

        quotes_with_tags.push(QuoteWithTags {
            id: quote.id,
            quote: quote.quote,
            source: quote.source,
            created_at: quote.created_at,
            updated_at: quote.updated_at,
            tags: tag_names,
        });
    }

    Ok(quotes_with_tags)
}

// Function to get a random quote with its tags
pub async fn get_random_quote(pool: &Pool<Sqlite>) -> Result<Option<QuoteWithTags>, sqlx::Error> {
    // Count total quotes
    let count = sqlx::query!("SELECT COUNT(*) as count FROM quotes")
        .fetch_one(pool)
        .await?
        .count;

    if count == 0 {
        return Ok(None);
    }

    // Get random quote
    let quote = sqlx::query_as!(
        Quote,
        "SELECT id, quote, source, created_at as \"created_at: DateTime<Utc>\", updated_at as \"updated_at: DateTime<Utc>\" FROM quotes ORDER BY RANDOM() LIMIT 1"
    )
        .fetch_optional(pool)
        .await?;

    match quote {
        Some(quote) => {
            // Get tags for this quote
            let tags = sqlx::query_as!(
                Tag,
                "SELECT id, quote_id, name, created_at as \"created_at: DateTime<Utc>\", updated_at as \"updated_at: DateTime<Utc>\" FROM tags WHERE quote_id = ?",
                quote.id
            )
                .fetch_all(pool)
                .await?;

            // Extract tag names
            let tag_names = tags.into_iter().map(|t| t.name).collect();

            Ok(Some(QuoteWithTags {
                id: quote.id,
                quote: quote.quote,
                source: quote.source,
                created_at: quote.created_at,
                updated_at: quote.updated_at,
                tags: tag_names,
            }))
        }
        None => Ok(None),
    }
}
