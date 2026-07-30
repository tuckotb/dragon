KERNEL := target/aarch64-unknown-none/debug/dragon
GDB := aarch64-elf-gdb
QEMU := qemu-system-aarch64

all: build debug

build:
	cargo build

clean:
	cargo clean

debug: build
	$(QEMU) \
		-M virt \
		-cpu cortex-a57 \
		-smp 4 \
		-m 4G \
		-nographic \
		-d int \
		-s -S \
		-kernel $(KERNEL) & \
	QEMU_PID=$$!; \
	trap 'kill $$QEMU_PID 2>/dev/null' EXIT INT TERM; \
	$(GDB) $(KERNEL) -x gdb.commands.txt

clippy:
	cargo clippy