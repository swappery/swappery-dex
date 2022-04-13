ALL_CONTRACTS = swappery-pair
CONTRACT_TARGET_DIR = target/wasm32-unknown-unknown/release

prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release --target wasm32-unknown-unknown $(patsubst %, -p %, $(ALL_CONTRACTS))
	$(foreach WASM, $(ALL_CONTRACTS), wasm-strip $(CONTRACT_TARGET_DIR)/$(subst -,_,$(WASM)).wasm 2>/dev/null | true;)

test: build-contract
	mkdir -p testing/test-pair/wasm
	cp target/wasm32-unknown-unknown/release/swappery_pair.wasm tests/test-pair/wasm
	cargo test

clippy:
	cd contract && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contract && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd contract && cargo fmt
	cd tests && cargo fmt

clean:
	cd contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
