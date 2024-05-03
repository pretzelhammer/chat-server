default:
  @just --list

server:
    RUSTFLAGS="-C target-cpu=native" RUST_LOG="info" cargo run --release --bin chat-server

debug-server:
    RUST_LOG="debug" cargo run --bin chat-server

build-server:
    RUSTFLAGS="-C target-cpu=native" cargo build --release --bin chat-server

build-debug-server:
    cargo build --bin chat-server

chat:
    RUSTFLAGS="-C target-cpu=native" RUST_LOG="info" cargo run --release --bin chat-tui

debug-chat:
    RUST_LOG="debug" cargo run --bin chat-tui

build-chat:
    RUSTFLAGS="-C target-cpu=native" cargo build --release --bin chat-tui

build-debug-chat:
    cargo build --bin chat-tui

bots:
    RUSTFLAGS="-C target-cpu=native" RUST_LOG="info" cargo run --release --bin chat-bots

debug-bots:
    RUST_LOG="debug" cargo run --bin chat-bots

build-bots:
    RUSTFLAGS="-C target-cpu=native" cargo build --release --bin chat-bots

build-debug-bots:
    cargo build --bin chat-bots

clear-logs:
    rm -rf logs

telnet:
    telnet 127.0.0.1 42069

diff EXAMPLE1 EXAMPLE2:
    diff -u --color examples/server-{{EXAMPLE1}}.rs examples/server-{{EXAMPLE2}}.rs

example NUMBER:
    RUST_LOG="debug" cargo run --example server-{{NUMBER}}
