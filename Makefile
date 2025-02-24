default:
	@grep '^[^#[:space:].].*:' Makefile

docker-start:
	docker compose up -d

docker-stop:
	docker compose stop

setup:
	

dev: start
start:
	cargo run
