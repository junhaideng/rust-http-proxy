test:
	cargo test && cd proxy && cargo test && cd ..

build:
	cargo build --release

fmt: 
	cargo fix --allow-dirty && cargo fmt && cd proxy && cargo fix --allow-dirty && cargo fmt && cd ..