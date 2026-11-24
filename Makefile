# Zevrigo Demo (MOD-18). Crate inti dari ../zevrigo (AGENDA P3).
.PHONY: fmt fmt-check lint check-forbidden test deps check-all docker-build app-web-check app-check

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

# Aplikasi Zevrigo Node (DEC-44, RENCANA_T16): antarmuka (npm ci tanpa skrip pasang, typecheck, test, build ke app/dist), lalu
# backend Tauri di workspace Cargo sendiri dengan Cargo.lock sendiri. Butuh sidecar di app/src-tauri/binaries/ dan WebKitGTK 4.1 di
# Linux. Larangan f32/f64/unsafe sama dengan repo ini.
app-web-check:
	cd app && npm ci --ignore-scripts --no-audit --no-fund && npm run check

app-check: app-web-check
	cd app/src-tauri && cargo fmt -- --check && cargo clippy --locked --all-targets -- -D warnings && bash ../../scripts/check-forbidden.sh && cargo test --locked
