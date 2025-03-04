# LedgerBeetle

> Combining the super powers of [TigerBeetle](https://tigerbeetle.com/) with the simplicity of [hledger](https://hledger.org/)

## Screenshots

![screenshot1.png](https://s6.imgcdn.dev/YhRft0.png)

## Development

**Requirements**

- rustup, cargo
- postgres-client
- docker
- bun
- make

**Setup**

Edit the `docker-compose.dev.yml` under the tb service, follow the changes shown below:

```diff
+++command: format --cluster=0 --replica=0 --replica-count=1 /data/0_0.tigerbeetle
+++# command: start --addresses=0.0.0.0:3001 /data/0_0.tigerbeetle
---# command: format --cluster=0 --replica=0 --replica-count=1 /data/0_0.tigerbeetle
---command: start --addresses=0.0.0.0:3001 /data/0_0.tigerbeetle
```

Then run `docker compose up tb` then when it is running `ctrl` + `C` and revert the change.

**Run**

For running all processes at the same time run: `make dev`

To run them individually here are the commands:

```
# setup
make dev-docker-start
cd frontend; bun i


# then run the following in 2 different terminals
make dev-be-start
make dev-fe-start
```
