# POLICY_TYPE can be one of `gatekeeper`, `opa`, with different build process
POLICY_TYPE ?= gatekeeper
SOURCE_FILES := $(shell find . -type f -name '*.rego')


policy.wasm: $(SOURCE_FILES)
ifeq ($(POLICY_TYPE), gatekeeper)
	opa build -t wasm -e policy/violation -o bundle.tar.gz policy.rego
else ifeq ($(POLICY_TYPE), opa)
	opa build -t wasm -e policy/main utility/policy.rego -o bundle.tar.gz policy.rego
else
	@printf "Please assign POLICY_TYPE to either 'gatekeeper' or 'opa'\n"
	exit 1
endif
	tar xvf bundle.tar.gz /policy.wasm
	rm bundle.tar.gz
	touch policy.wasm # opa creates the bundle with unix epoch timestamp, fix it

.PHONY: test
test:
	opa test policy.rego tests/*.rego


annotated-policy.wasm: policy.wasm metadata.yml
	kwctl annotate -m metadata.yml -u README.md -o annotated-policy.wasm policy.wasm

.PHONY: e2e-tests
e2e-tests: annotated-policy.wasm
	# bats e2e.bats
	# no-op

.PHONY: clean
clean:
	rm -f *.wasm *.tar.gz
