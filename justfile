# list commands
default:
    @just --list

# list commands
list:
    @just --list

# run prod server
server:
    RUSTFLAGS="-C target-cpu=native" RUST_LOG="info" cargo run --release --bin chat-server

# run debug server
debug-server:
    RUST_LOG="debug" cargo run --bin chat-server

# build prod server
build-server:
    RUSTFLAGS="-C target-cpu=native" cargo build --release --bin chat-server

# build debug server
build-debug-server:
    cargo build --bin chat-server

# run prod TUI chat client
chat:
    RUSTFLAGS="-C target-cpu=native" RUST_LOG="info" cargo run --release --bin chat-tui

# run debug TUI chat client
debug-chat:
    RUST_LOG="debug" cargo run --bin chat-tui

# build prod TUI chat client
build-chat:
    RUSTFLAGS="-C target-cpu=native" cargo build --release --bin chat-tui

# build debug TUI chat client
build-debug-chat:
    cargo build --bin chat-tui

# run prod chat bots
bots:
    RUSTFLAGS="-C target-cpu=native" RUST_LOG="info" cargo run --release --bin chat-bots

# run debug chat bots
debug-bots:
    RUST_LOG="debug" cargo run --bin chat-bots

# build prod chat bots
build-bots:
    RUSTFLAGS="-C target-cpu=native" cargo build --release --bin chat-bots

# build debug chat bots
build-debug-bots:
    cargo build --bin chat-bots

# delete logs directory
clean-logs:
    rm -rf logs

# connect to server with telnet
telnet:
    telnet 127.0.0.1 42069

# diff two examples
diff EXAMPLE1 EXAMPLE2:
    diff -u --color examples/server-{{EXAMPLE1}}.rs examples/server-{{EXAMPLE2}}.rs

# run example
example NUMBER:
    RUST_LOG="debug" cargo run --example server-{{NUMBER}}
