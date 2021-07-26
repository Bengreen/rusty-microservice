CARGO:=cargo

build:
	cargo build

release:
	cargo build --release


status:
	@${CARGO} --version

test:
	cargo test

run:
	cargo run -- listen
