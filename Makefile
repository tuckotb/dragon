all: build run

build:
	cargo build

clean:
	cargo clean

run:
	qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a57 \
	-smp 4 \
	-m 4G \
    -nographic \
    -kernel target/aarch64-unknown-none/debug/dragon

clippy:
	cargo clippy