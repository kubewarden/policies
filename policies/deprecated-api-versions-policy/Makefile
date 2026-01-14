
policy.wasm: 
	$(MAKE) --file ../Makefile.rust $@

annotated-policy.wasm: policy.wasm metadata.yml
	$(MAKE) --file ../Makefile.rust $@

.PHONY: fmt
fmt:
	$(MAKE) --file ../Makefile.rust $@

.PHONY: lint
lint:
	$(MAKE) --file ../Makefile.rust $@

.PHONY: e2e-tests
e2e-tests: annotated-policy.wasm
	$(MAKE) --file ../Makefile.rust $@

.PHONY: test
test: fmt lint
	$(MAKE) --file ../Makefile.rust $@

.PHONY: clean
clean:
	$(MAKE) --file ../Makefile.rust $@

.PHONY: expected-policy-metadata
expected-policy-metadata:
	cargo --locked run --quiet --package policy-metadata-helper -- --metadata-path metadata.yml build

.PHONY: check-policy-metadata
check-policy-metadata:
	cargo --locked run --quiet --package policy-metadata-helper -- --metadata-path metadata.yml check
