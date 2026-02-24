# cel-policy specific build flags
EXTRA_GCFLAGS = -l -wb=false

# E2E test metadata file
E2E_TEST_ANNOTATION_METADATA ?= test_data/annotation-metadata.yml

# Include shared wasigo Makefile
include ../Makefile.wasigo

# Override e2e-tests with cel-specific metadata merging
.PHONY: e2e-tests
e2e-tests: policy.wasm
	# The policy annotation uses a different metadata file for testing because the 
	# e2e tests require access to ConfigMap to test the parameters feature.
	yq eval-all 'select(fileIndex == 0) *+? select(fileIndex == 1)' metadata.yml test_data/metadata.yml > $(E2E_TEST_ANNOTATION_METADATA)
	kwctl annotate -m $(E2E_TEST_ANNOTATION_METADATA) -u README.md -o annotated-policy.wasm policy.wasm
	bats e2e.bats
