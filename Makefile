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
	-d int \
	-S \
	-gdb tcp::1234 \
    -kernel target/aarch64-unknown-none/debug/dragon

clippy:
	cargo clippy