#
# Makefile for katherine fundraising
#

ifndef NEAR_ACCOUNT
NEAR_ACCOUNT="huxley.testnet"
endif

lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Build library dynamically linked to the rust runtime libraries
build:
	echo "Building katherine fundrising"
	RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/katherine_fundraising_contract.wasm res/

publish-dev: build
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm

publish-dev-init: build
	rm -rf neardev/
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": ${NEAR_ACCOUNT}, "min_deposit_amount": 2, "metapool_contract_address": "meta-v2.pool.testnet", "katherine_fee_percent": 100 }'

integration: build
	scripts/integration.sh

install:
	cp target/release/libcfdi.so /usr/local/lib64/

test:
	# TODO: create container for database
	RUST_BACKTRACE=1 cargo test "${TEST_PREFIX}" -- --color always --nocapture

format:
	cargo fmt -- --check

doc:
	cargo doc

clean:
	cargo clean
