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

docker:
	docker build -t rust_hello .

docker-shell: docker
	docker run -it rust_hello

docker-tag: docker
	docker tag rust_hello:latest rust_hello:1.0.0

rollout:
	kubectl rollout restart deployment hello

bloat:
	cargo bloat --release -n 10

docs:
	@cargo doc --no-deps

style-check:
	cargo fmt --all -- --check