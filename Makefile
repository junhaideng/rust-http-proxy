test:
	cargo test && cd proxy && cargo test && cd ..

build:
	cargo build --release