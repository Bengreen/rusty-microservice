CARGO:=cargo

IMAGE_NAME=rusty-microservice

build:
	cargo build

release:
	cargo build --release


status:
	@cargo --version

test:
	cargo test -- --nocapture

run:
	cargo run -- listen

docker:
	docker build -t $(IMAGE_NAME) .

docker-shell: docker
	docker run -it $(IMAGE_NAME)

docker-tag: docker
	docker tag rust_hello:latest rust_hello:1.0.0

rollout:
	kubectl rollout restart deployment hello

bloat:
	cargo bloat --release -n 10

doc:
	@cargo doc --no-deps --open
doc-watch:
	@cargo watch -x 'doc --no-deps --workspace --open'

style-check:
	@cargo fmt --all -- --check

lint:
	@cargo clippy

benchmark:
	@cargo criterion
	@open target/criterion/reports/index.html
