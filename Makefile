#
# Makefile for katherine fundraising
#

YOCTO_UNITS=000000000000000000000000

ifndef NEAR_ACCOUNT
NEAR_ACCOUNT="kate_tester3.testnet"
endif

lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Build library dynamically linked to the rust runtime libraries
build:
	echo "Building katherine fundrising"
	RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/katherine_fundraising_contract.wasm res/
	cp target/wasm32-unknown-unknown/release/test_meta_pool.wasm res/
	cp target/wasm32-unknown-unknown/release/test_p_token.wasm res/

publish-dev: build
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm

publish-dev-init: build
	rm -rf neardev/
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": ${NEAR_ACCOUNT}, "min_deposit_amount": "2000000000000", "metapool_contract_address": "meta-v2.pool.testnet", "katherine_fee_percent": 100 }'

integration-meta-pool: build
	./scripts/integration_meta_pool.sh

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
