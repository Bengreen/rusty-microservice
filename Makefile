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

kubectl-restart:
	kubectl rollout restart deployment hello