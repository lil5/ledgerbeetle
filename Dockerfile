FROM oven/bun AS bun

WORKDIR /app

COPY ./frontend/package.json ./frontend/bun.lock ./

RUN bun install --frozen-lockfile

COPY ./frontend/ .

ENV VITE_ALLOW_ADD=true

RUN bun run build

FROM caddy:2 AS caddy

FROM rust:bookworm AS rust

RUN apt-get update && apt install -y curl postgresql-client-15 libclang-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN cargo fetch --locked

COPY . .

RUN cargo build --release

COPY --from=caddy /usr/bin/caddy /usr/bin/

COPY ./frontend/Caddyfile /etc/caddy/

COPY --from=bun /app/dist/ /srv/

EXPOSE 8080

CMD [ "/bin/bash", "-c" ,"/app/target/release/app & caddy run --config /etc/caddy/Caddyfile & wait -n; exit $?"]
