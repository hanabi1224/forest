SER_TESTS = "tests/serialization_tests"

install-cli:
	cargo install --locked --path forest/cli --force

install-daemon:
	cargo install --locked --path forest/daemon --force

install: install-cli install-daemon

clean-all:
	cargo clean

clean:
	@echo "Cleaning local packages..."
	@cargo clean -p forest-cli
	@cargo clean -p forest-daemon
	@cargo clean -p forest_cli_shared
	@cargo clean -p forest_libp2p
	@cargo clean -p forest_blocks
	@cargo clean -p forest_chain_sync
	@cargo clean -p forest_message
	@cargo clean -p forest_state_manager
	@cargo clean -p forest_interpreter
	@cargo clean -p forest_crypto
	@cargo clean -p forest_encoding
	@cargo clean -p forest_ipld
	@cargo clean -p forest_legacy_ipld_amt
	@cargo clean -p forest_json
	@cargo clean -p forest_fil_types
	@cargo clean -p forest_rpc
	@cargo clean -p forest_key_management
	@cargo clean -p forest_utils
	@cargo clean -p forest_test_utils
	@cargo clean -p forest_message_pool
	@cargo clean -p forest_genesis
	@cargo clean -p forest_actor_interface
	@cargo clean -p forest_networks
	@echo "Done cleaning."

# Lints with everything we have in our CI arsenal
lint-all: lint audit udeps spellcheck

audit:
	cargo audit --ignore RUSTSEC-2020-0071 --ignore RUSTSEC-2022-0040

udeps:
	cargo udeps --all-targets --features submodule_tests

spellcheck:
	cargo spellcheck --code 1

lint: license clean
	cargo fmt --all --check
	taplo fmt --check
	taplo lint
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets --features forest_deleg_cns -- -D warnings

# Formats Rust and TOML files
fmt:
	cargo fmt --all
	taplo fmt

build:
	cargo build --bin forest --bin forest-cli

release:
	cargo build --release --bin forest --bin forest-cli

docker-run:
	docker build -t forest:latest -f ./Dockerfile . && docker run forest

# Git submodule test vectors
pull-serialization-tests:
	git submodule update --init

run-serialization-vectors:
	cargo nextest run --release --manifest-path=$(SER_TESTS)/Cargo.toml --features submodule_tests

run-vectors: run-serialization-vectors

test-vectors: pull-serialization-tests run-vectors

# Test all without the submodule test vectors with release configuration
test:
	cargo nextest run --all --exclude serialization_tests --exclude forest_message --exclude forest_crypto
	cargo nextest run -p forest_crypto --features blst --no-default-features
	cargo nextest run -p forest_message --features blst --no-default-features

test-release:
	cargo nextest run --release --all --exclude serialization_tests --exclude forest_message --exclude forest_crypto
	cargo nextest run --release -p forest_crypto --features blst --no-default-features
	cargo nextest run --release -p forest_message --features blst --no-default-features

smoke-test:
	./scripts/smoke_test.sh

test-all: test-release test-vectors

# Checks if all headers are present and adds if not
license:
	./scripts/add_license.sh

docs:
	cargo doc --no-deps

mdbook:
	mdbook serve documentation

mdbook-build:
	mdbook build ./documentation

rustdoc:
	cargo doc --workspace --all-features --no-deps

.PHONY: clean clean-all lint build release test test-all test-release license test-vectors run-vectors pull-serialization-tests install-cli install-daemon install docs run-serialization-vectors rustdoc
