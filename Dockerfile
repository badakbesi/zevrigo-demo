# syntax=docker/dockerfile:1
# Paket Docker Zevrigo Coin, hanya regtest (TSK-M33, Spesifikasi Docker v0.1, DEC-10 bagian 9).
# Satu image: zevrigo-node, zevrigo-miner, zevrigo-cli (repo inti zevrigo), zevrigo-explorer + antarmuka web (repo
# zevrigo-explorer), zevrigo-demo (layanan jaringan demo, DEC-36, repo ini), dan blog statis (repo zevrigo-blog, DEC-37).
# Image dasar dikunci digest indeks (registry resmi Docker Hub), sehingga build ulang memakai dasar yang sama.
#
# Sejak 18 November 2026 (DEC-39) berkas ini ada di repo zevrigo-demo. Konteks build adalah folder induk zevrigo_lab (repo
# zevrigo, zevrigo-explorer, zevrigo-demo, dan zevrigo-blog berdampingan); yang masuk konteks hanya daftar izin di
# Dockerfile.dockerignore (dibaca BuildKit). Dari folder zevrigo-demo:
#
#   make docker-build             (docker build -f Dockerfile -t zevrigo:regtest ..)
#   docker compose build          (compose.yaml: context .., dockerfile zevrigo-demo/Dockerfile)

ARG RUST_IMAGE=rust:1.99.0-slim-trixie@sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581
ARG NODE_IMAGE=node:24.22.0-trixie-slim@sha256:a6155384446670e9719bdc4fd0ebe4d4b29fb32a8d6aa523da3547a6cfbcf22c
ARG RUNTIME_IMAGE=debian:trixie-slim@sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4

# 1. Program Rust (release, Cargo.lock tiap repo dikunci). Tiga workspace: repo inti (node, miner, CLI), zevrigo-explorer
#    (crate inti dan randomx-rs salinan ../zevrigo/vendor, DEC-01), dan zevrigo-demo; hasil build bersama di /src/target.
#    rust-toolchain.toml dan .cargo tidak masuk konteks: versi Rust dari image.
FROM ${RUST_IMAGE} AS rust-build
# RandomX (DEC-01, DEC-10 bagian 10): randomx-rs membangun kode C++ resmi lewat cmake, make, dan g++
# (image rust slim tidak berisi make).
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake make g++ \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY zevrigo ./zevrigo
COPY zevrigo-explorer/Cargo.toml zevrigo-explorer/Cargo.lock ./zevrigo-explorer/
COPY zevrigo-explorer/src ./zevrigo-explorer/src
COPY zevrigo-demo/Cargo.toml zevrigo-demo/Cargo.lock ./zevrigo-demo/
COPY zevrigo-demo/src ./zevrigo-demo/src
# Penjaga daftar izin: bila Dockerfile.dockerignore tidak dipakai, konteks memuat seluruh repo dan build berhenti di sini.
RUN for f in zevrigo/rust-toolchain.toml zevrigo/.cargo zevrigo/target zevrigo/docs zevrigo/.git; do \
      if [ -e "$f" ]; then echo "konteks build memuat $f: Dockerfile.dockerignore tidak dipakai" >&2; exit 1; fi; \
    done
# Penjaga sebaliknya: modul Rust bernama target/out tetap ikut (Q151 percobaan 1).
RUN test -f zevrigo/crates/pow/src/target/mod.rs || { echo "konteks build tidak memuat crates/pow/src/target" >&2; exit 1; }
ENV CARGO_TARGET_DIR=/src/target
RUN cd zevrigo && cargo build --release --locked -p zevrigo-node -p zevrigo-miner -p zevrigo-cli
RUN cd zevrigo-explorer && cargo build --release --locked --bin zevrigo-explorer
RUN cd zevrigo-demo && cargo build --release --locked --bin zevrigo-demo
RUN mkdir /out \
 && cp target/release/zevrigo-node target/release/zevrigo-explorer \
       target/release/zevrigo-miner target/release/zevrigo-cli target/release/zevrigo-demo /out/

# 2. Antarmuka web explorer (Next.js static export, repo zevrigo-explorer): paket dari package-lock.json tanpa skrip pasang.
FROM ${NODE_IMAGE} AS web-build
WORKDIR /web
COPY zevrigo-explorer/web/package.json zevrigo-explorer/web/package-lock.json ./
RUN npm ci --ignore-scripts --no-audit --no-fund
COPY zevrigo-explorer/web ./
RUN npm run build

# 3. Blog statis (DEC-37, TSK-M64, repo zevrigo-blog): paket sama persis dengan antarmuka explorer, artikel Markdown dibaca
#    saat build.
FROM ${NODE_IMAGE} AS blog-build
WORKDIR /blog
COPY zevrigo-blog/package.json zevrigo-blog/package-lock.json ./
RUN npm ci --ignore-scripts --no-audit --no-fund
COPY zevrigo-blog ./
RUN npm run build

# 4. Runtime: Debian slim, pengguna biasa (uid 10001), data di volume /data.
FROM ${RUNTIME_IMAGE}
RUN groupadd --system --gid 10001 zevrigo \
 && useradd --system --uid 10001 --gid zevrigo --home-dir /data --no-create-home --shell /usr/sbin/nologin zevrigo \
 && mkdir -p /data /usr/share/zevrigo \
 && chown zevrigo:zevrigo /data
COPY --from=rust-build /out/ /usr/local/bin/
COPY --from=web-build /web/out /usr/share/zevrigo/web
COPY --from=blog-build /blog/out /usr/share/zevrigo/blog
USER zevrigo
WORKDIR /data
VOLUME ["/data"]
ENTRYPOINT ["zevrigo-node"]
CMD ["--regtest", "--datadir", "/data"]
