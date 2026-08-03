KERNEL := target/aarch64-unknown-none/debug/dragon
GDB := aarch64-elf-gdb
QEMU := qemu-system-aarch64

all: build

build:
	cargo build

clean:
	cargo clean

clippy:
	cargo clippy
