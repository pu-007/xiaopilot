linux-target := "x86_64-unknown-linux-musl"
win-target := "x86_64-pc-windows-msvc"

# build both binaries → release/
default: build-all

# install prerequisites (run once)
setup:
    rustup target add {{ linux-target }}
    rustup target add {{ win-target }}
    @echo Prerequisites checked. If zig or cargo-zigbuild are missing, install them:
    @echo   scoop install zig
    @echo   cargo install cargo-zigbuild

# build both
build-all: build-linux build-win

# build xiaopilot-wol → release/xiaopilot-wol/ + release_private/xiaopilot-wol/
build-linux:
    cargo zigbuild --release --target {{ linux-target }} -p xiaopilot-wol
    mkdir -p release/xiaopilot-wol
    cp target/{{ linux-target }}/release/xiaopilot-wol release/xiaopilot-wol/
    cp .env.example release/xiaopilot-wol/.env
    cp wol.yml.example release/xiaopilot-wol/wol.yml
    cp scripts/start-xiaopilot-wol.sh release/xiaopilot-wol/
    mkdir -p release_private/xiaopilot-wol
    cp target/{{ linux-target }}/release/xiaopilot-wol release_private/xiaopilot-wol/
    cp scripts/start-xiaopilot-wol.sh release_private/xiaopilot-wol/

# build xiaopilot-win → release/xiaopilot-win/ + release_private/xiaopilot-win/
build-win:
    cargo build --release --target {{ win-target }} -p xiaopilot-win
    mkdir -p release/xiaopilot-win
    cp target/{{ win-target }}/release/xiaopilot-win.exe release/xiaopilot-win/
    cp .env.example release/xiaopilot-win/.env
    cp win.yml.example release/xiaopilot-win/win.yml
    cp scripts/start-xiaopilot.vbs release/xiaopilot-win/
    mkdir -p release_private/xiaopilot-win
    cp target/{{ win-target }}/release/xiaopilot-win.exe release_private/xiaopilot-win/
    cp scripts/start-xiaopilot.vbs release_private/xiaopilot-win/

# remove build artifacts
clean:
    cargo clean
