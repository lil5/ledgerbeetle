default:
	@grep '^[^#[:space:].].*:' Makefile

dev-docker-start:
	docker compose -f docker-compose.dev.yml up -d
dev-docker-stop:
	docker compose -f docker-compose.dev.yml stop

dev-be-start:
	cargo run
dev-fe-start:
	cd frontend; bun run dev

prod-start:
	docker compose -f docker-compose.prod.yml up -d
prod-stop:
	docker compose -f docker-compose.prod.yml stop

dev:
	cd frontend; bun i
	make dev-docker-start 
	make dev-be-start & make dev-fe-start & wait
start: dev
