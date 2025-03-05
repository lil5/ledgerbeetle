FROM rust:bookworm

RUN apt-get update && apt install -y curl postgresql-client-15 libclang-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN cargo fetch --locked

COPY . .

RUN cargo build --release

EXPOSE 3000

ENTRYPOINT [ "/app/target/release/app" ]