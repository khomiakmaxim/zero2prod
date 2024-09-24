# TODO: Use semantic versioning here
FROM rust:1.81.0
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN chmod +x scripts/init_db.sh
RUN scripts/init_db.sh
RUN cargo build --release
ENTRYPOINT ["./target/release/zero2prod"]