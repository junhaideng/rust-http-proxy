test:
	cargo test && cd proxy && cargo test && cd ..

build:
	cargo build --release

fmt: 
	cargo fix --allow-dirty --allow-staged && cargo fmt && cd proxy && cargo fix --allow-dirty --allow-staged && cargo fmt && cd .. 

cloc:
	cloc . --exclude-dir=target, proxy/target, .vscode, .idea

curl:
	curl -I http://httpbin.org -v -x 127.0.0.1:8080 --proxy-user rust:proxy
