# Zevrigo Demo (MOD-18). Crate inti dari ../zevrigo (AGENDA P3).
.PHONY: fmt fmt-check lint check-forbidden test deps check-all docker-build

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

check-forbidden:
	bash scripts/check-forbidden.sh

test:
	cargo test --workspace

deps:
	cargo tree --workspace --depth 1 -e normal

check-all: fmt-check lint check-forbidden test

# Paket Docker regtest (TSK-M33, Spesifikasi Docker v0.1): image zevrigo:regtest berisi node, CLI, dan miner (repo inti),
# explorer + antarmuka web (zevrigo-explorer), zevrigo-demo, dan blog (zevrigo-blog). Konteks build folder induk zevrigo_lab;
# yang masuk konteks diatur Dockerfile.dockerignore.
docker-build:
	docker build -f Dockerfile -t zevrigo:regtest ..
