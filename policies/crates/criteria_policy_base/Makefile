CARGO_GLOBAL_OPTIONS ?= --locked

.PHONY: build
build:
	cargo $(CARGO_GLOBAL_OPTIONS) build --release

.PHONY: fmt
fmt:
	cargo $(CARGO_GLOBAL_OPTIONS) fmt --all -- --check

.PHONY: lint
lint:
	cargo $(CARGO_GLOBAL_OPTIONS) clippy --no-default-features --features "annotations" -- -D warnings

.PHONY: check
check:
	@for feature in $$(cargo $(CARGO_GLOBAL_OPTIONS) read-manifest --manifest-path Cargo.toml | jq -r '.features | keys[]'); do \
		cargo $(CARGO_GLOBAL_OPTIONS) check --no-default-features --features "$$feature"; \
	done
	
.PHONY: typos
typos:
	# run typo checker from crate-ci/typos
	typos $(realpath $(dir $(lastword $(MAKEFILE_LIST))))

.PHONY: test
test: fmt lint
	cargo $(CARGO_GLOBAL_OPTIONS) test 

.PHONY: clean
clean:
	cargo $(CARGO_GLOBAL_OPTIONS) clean
