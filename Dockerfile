FROM lukemathwalker/cargo-chef:latest-rust-1.81.0 AS builder
WORKDIR /app
RUN apt update && apt install lld clang -y

# FROM builder AS intermediate
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# FROM builder AS planner
# COPY --from=intermediate /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path /app/recipe.json
ENV SQLX_OFFLINE=true

# CHECK: why caching becomes broken if to remove below `COPY` command?
# Isn't the above `COPY` command enough here?
COPY . .
RUN cargo build --release --bin zero2prod


FROM debian:bookworm-slim AS runtime
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./zero2prod"] 