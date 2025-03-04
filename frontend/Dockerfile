FROM oven/bun AS build

WORKDIR /app

COPY package.json bun.lock ./

RUN bun install --frozen-lockfile

COPY . .

RUN bun run build

FROM caddy AS caddy

COPY Caddyfile /etc/caddy/

COPY --from=build /app/dist/ /srv/

ENTRYPOINT [ "caddy","start" ]
