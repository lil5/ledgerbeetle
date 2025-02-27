default:
	@grep '^[^#[:space:].].*:' Makefile

docker-start:
	docker compose up -d

docker-stop:
	docker compose stop

setup:
	touch .hledger.journal

dev: start
start:
	cargo run

hledger-start:
	hledger-web --serve --rules=view,add,manage -f .hledger.journal