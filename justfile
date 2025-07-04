dev-docker-start:
	docker compose -f docker-compose.dev.yml up -d
dev-docker-stop:
	docker compose -f docker-compose.dev.yml stop

dev-be-start:
	cargo run
dev-fe-start:
	cd frontend; VITE_ALLOW_ADD=true pnpm run dev

prod-start:
	docker compose -f docker-compose.prod.yml up -d
prod-stop:
	docker compose -f docker-compose.prod.yml stop
prod-build:
	docker compose -f docker-compose.prod.yml build

dev:
	cd frontend; pnpm install
	just dev-docker-start 
	just dev-be-start & just dev-fe-start & wait
start: dev

gen-openapi:
	wget -O openapi.json http://localhost:5173/api/openapi