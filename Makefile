default:
	@grep '^[^#[:space:].].*:' Makefile

dev-docker-start:
	docker compose -f docker-compose.dev.yml up -d
dev-docker-stop:
	docker compose -f docker-compose.dev.yml stop

dev-bin-start:
	cargo run

prod-start:
	docker compose -f docker-compose.prod.yml up -d
prod-stop:
	docker compose -f docker-compose.prod.yml stop

dev: dev-docker-start dev-bin-start
start: dev
