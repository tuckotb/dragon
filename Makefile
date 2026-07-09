all: build run

build:
	cargo build

clean:
	cargo clean

run:
	qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a57 \
    -nographic \
    -kernel target/aarch64-unknown-none/debug/arm-kernel

clippy:
	cargo clippy