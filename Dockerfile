# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.87

# Build stage
FROM rust:${RUST_VERSION} AS build
WORKDIR /build

# Install dependencies
RUN apt-get update && apt-get install -y git curl sqlite3 && \
    rm -rf /var/lib/apt/lists/*

# Set DATABASE_URL for sqlx compile-time verification
ENV DATABASE_URL=sqlite:///build/db/quotes.db

# Create database directory and initialize with migrations
RUN mkdir -p /build/db && \
    sqlite3 /build/db/quotes.db "SELECT 1;"

# Copy migration files and run them
COPY migrations ./migrations
RUN sqlite3 /build/db/quotes.db < migrations/20250425230811_create_quotes.up.sql && \
    sqlite3 /build/db/quotes.db < migrations/20250425231048_create_tags.up.sql

# Build application with cached dependencies
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=askama.toml,target=askama.toml \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=cache,target=/build/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && \
    cp target/release/quote-server /bin/quote-server

# Run the application once to initialize the database with migrations and default data
RUN --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=migrations,target=migrations \
    timeout 10s /bin/quote-server || true

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    appuser

# Copy the binary and database from build stage
COPY --from=build /bin/quote-server /bin/quote-server
COPY --from=build /build/db/quotes.db /app/default-quotes.db

# Create startup script
RUN echo '#!/bin/bash\n\
if [ ! -f /app/db/quotes.db ]; then\n\
  echo "Database not found, copying default database..."\n\
  cp /app/default-quotes.db /app/db/quotes.db\n\
fi\n\
exec /bin/quote-server "$@"' > /app/start.sh && \
chmod +x /app/start.sh

# Create app directory and set ownership
RUN mkdir -p /app/db && chown -R appuser:appuser /app

USER appuser
WORKDIR /app

# Copy necessary assets
COPY --chown=appuser:appuser assets ./assets
COPY --chown=appuser:appuser credentials.txt ./credentials.txt

# Set DATABASE_URL for runtime
ENV DATABASE_URL=sqlite:///app/db/quotes.db

# Expose application port
EXPOSE 3000

# Run the application via startup script
CMD ["/app/start.sh"]