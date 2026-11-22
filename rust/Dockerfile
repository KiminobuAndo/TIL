# Rust 学習用イメージ
# 公式イメージは --profile minimal でツールチェーンを入れているため、
# clippy / rustfmt は同梱されていない。ここで明示的に追加する。
FROM rust:1-slim-bookworm

# 学習中に使う最小限のツール
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        git \
        less \
        libssl-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

# ログインシェル（bash -l）は /etc/profile で PATH を上書きしてしまい
# cargo が見えなくなるため、profile.d で補っておく
RUN printf 'export PATH="%s/bin:$PATH"\n' "${CARGO_HOME}" > /etc/profile.d/cargo.sh

# ホストと同じ uid/gid のユーザーで動かす。
# root で動かすとコンテナ内の cargo new が bind mount 上に root 所有の
# ファイルを作り、ホスト側から編集できなくなる。
ARG UID=1000
ARG GID=1000
RUN groupadd --gid "${GID}" rust \
    && useradd --uid "${UID}" --gid "${GID}" --create-home --shell /bin/bash rust \
    && mkdir -p "${CARGO_HOME}/registry" /cargo-target \
    && chown -R "${UID}:${GID}" "${CARGO_HOME}" /cargo-target

# ビルド成果物を bind mount の外に出す（ホストのリポジトリを汚さない / ビルドが速い）
ENV CARGO_TARGET_DIR=/cargo-target

USER rust
WORKDIR /workspace
CMD ["bash"]
