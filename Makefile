.PHONY: kb

kb:
	cargo build --bin kb --release
	elf2uf2-rs ./target/thumbv6m-none-eabi/release/kb -ds
