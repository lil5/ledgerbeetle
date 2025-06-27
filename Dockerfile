FROM node:bookworm-slim AS node

RUN npm install -g pnpm@latest-10

WORKDIR /app

COPY ./frontend/package.json ./frontend/pnpm-lock.yaml ./

RUN --mount=type=cache,id=pnpm,target=/root/.local/share/pnpm/store pnpm install --frozen-lockfile

COPY ./frontend/ .

ENV VITE_ALLOW_ADD=true

RUN pnpm run build

FROM caddy:2 AS caddy

FROM rust:bookworm AS rust

RUN apt-get update && apt install -y curl postgresql-client-15 libclang-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN cargo fetch --locked

COPY . .

ARG CARGO_BUILD_JOBS=default

RUN  --mount=type=cache,target=/usr/local/cargo/registry \
   --mount=type=cache,target=/app/target \
   cargo build --release && cp /app/target/release/app /usr/bin/app

COPY --from=caddy /usr/bin/caddy /usr/bin/

COPY ./frontend/Caddyfile /etc/caddy/

COPY --from=node /app/dist/ /srv/

EXPOSE 8080

CMD [ "/bin/bash", "-c" ,"/usr/bin/app & caddy run --config /etc/caddy/Caddyfile & wait -n; exit $?"]
