linux-target := "x86_64-unknown-linux-musl"
win-target := "x86_64-pc-windows-msvc"
out-dir := "bin"

# build both binaries → bin/
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

# build xiaopilot-wol → bin/xiaopilot-wol
build-linux:
    cargo zigbuild --release --target {{ linux-target }} -p xiaopilot-wol
    mkdir -p {{ out-dir }}
    cp target/{{ linux-target }}/release/xiaopilot-wol {{ out-dir }}/

# build xiaopilot-win → bin/xiaopilot-win.exe
build-win:
    cargo build --release --target {{ win-target }} -p xiaopilot-win
    mkdir -p {{ out-dir }}
    cp target/{{ win-target }}/release/xiaopilot-win.exe {{ out-dir }}/

# remove bin/ and build artifacts
clean:
    cargo clean
    rm -rf {{ out-dir }}
