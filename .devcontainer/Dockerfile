# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.238.1/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
FROM mcr.microsoft.com/devcontainers/rust:latest

# Rust specific setup (install official toolchain so clippy can be added; image may ship a custom one)
RUN rustup toolchain uninstall 1.93.0 2>/dev/null || true \
    && rustup toolchain install 1.93.0 \
    && rustup default 1.93.0 \
    && rustup toolchain install nightly-x86_64-unknown-linux-gnu \
    && rustup component add clippy \
    && rustup component add --toolchain nightly-x86_64-unknown-linux-gnu rustfmt

# [Optional] Uncomment this section to install additional packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>

# make symlink to config file in devcontainer directory
RUN mkdir -p /home/vscode/.config/matrix-tui \
    && ln -s /workspaces/matrix-tui/.devcontainer/config.toml /home/vscode/.config/matrix-tui/config.toml
