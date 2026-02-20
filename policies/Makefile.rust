ROOT_DIR ?= $(dir $(realpath $(lastword $(MAKEFILE_LIST))))
POLICY_DIR := $(notdir $(patsubst %/,%,$(ROOT_DIR)))
TARGET_DIR ?= $(CURDIR)/target
CARGO_GLOBAL_OPTIONS ?= --locked

# Find all Rust source files to track as dependencies
RUST_SOURCES := $(shell find $(CURDIR)/src -name "*.rs" 2>/dev/null)

# Some projects do not have a Cargo.lock, hence we cannot force the presence of Cargo.lock in the Makefile dependencies. 
# Instead, we will track all Cargo.* files, which includes Cargo.toml and Cargo.lock if it exists.
CARGO_FILES := $(shell find $(CURDIR) -name "Cargo.*" 2>/dev/null)

policy.wasm: $(CARGO_FILES) $(RUST_SOURCES)
	cargo $(CARGO_GLOBAL_OPTIONS) build --target=wasm32-wasip1 --target-dir=$(TARGET_DIR) --release 
	cp $(TARGET_DIR)/wasm32-wasip1/release/*.wasm $(CURDIR)/policy.wasm

annotated-policy.wasm: policy.wasm metadata.yml
	kwctl annotate -m metadata.yml -u README.md -o $(CURDIR)/annotated-policy.wasm $(CURDIR)/policy.wasm

.PHONY: fmt
fmt:
	cargo $(CARGO_GLOBAL_OPTIONS) fmt --all -- --check

.PHONY: lint
lint:
	cargo $(CARGO_GLOBAL_OPTIONS) clippy -- -D warnings

.PHONY: e2e-tests
e2e-tests: annotated-policy.wasm
	bats e2e.bats

.PHONY: test
test: fmt lint
	cargo $(CARGO_GLOBAL_OPTIONS) test

.PHONY: clean
clean:
	cargo $(CARGO_GLOBAL_OPTIONS) clean
	rm -f policy.wasm annotated-policy.wasm

.PHONY: debug
debug:
	@echo $(MAKEFILE_LIST)
	@echo "ROOT_DIR=$(ROOT_DIR)"
	@echo "CURDIR=$(CURDIR)"
