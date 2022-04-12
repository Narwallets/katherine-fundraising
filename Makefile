#
# Makefile for katherine fundraising
#

YOCTO_UNITS=000000000000000000000000

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
	cp target/wasm32-unknown-unknown/release/test_meta_pool.wasm res/

publish-dev: build
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm

publish-dev-init: build
	rm -rf neardev/
	NEAR_ENV=testnet near dev-deploy --wasmFile res/katherine_fundraising_contract.wasm --initFunction new --initArgs '{"owner_id": ${NEAR_ACCOUNT}, "min_deposit_amount": 2, "metapool_contract_address": "meta-v2.pool.testnet", "katherine_fee_percent": 100 }'

publish-dev-meta-pool-init: build
	rm -rf neardev/
	rm -rf neardev_metapool/
	NEAR_ENV=testnet near dev-deploy --wasmFile res/test_meta_pool.wasm --initFunction new_default_meta --initArgs '{"owner_id": ${NEAR_ACCOUNT}, "total_supply": "1000${YOCTO_UNITS}" }'
	mv neardev/ neardev_metapool/
	NEAR_ACCOUNT=${NEAR_ACCOUNT} scripts/export_meta_pool.sh

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
