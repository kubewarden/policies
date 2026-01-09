.PHONY: clean annotated-policy.wasm test lint e2e-tests

# Helper function to run a target across all policies with summary
define run-target
	@passed=0; failed=0; failed_policies=""; \
	for policy in policies/*/; do \
		if [ -f "$$policy/Makefile" ]; then \
			echo "Running $(1) in $$policy"; \
			if $(MAKE) -C "$$policy" $(1); then \
				passed=$$((passed + 1)); \
			else \
				failed=$$((failed + 1)); \
				failed_policies="$$failed_policies  - $$policy\n"; \
			fi; \
		fi; \
	done; \
	echo ""; \
	echo "=== $(1) Summary ==="; \
	echo "Passed: $$passed"; \
	echo "Failed: $$failed"; \
	if [ $$failed -gt 0 ]; then \
		echo ""; \
		echo "Failed policies:"; \
		printf "$$failed_policies"; \
		exit 1; \
	fi
endef

clean:
	@for policy in policies/*/; do \
		if [ -f "$$policy/Makefile" ]; then \
			echo "Cleaning $$policy"; \
			$(MAKE) -C "$$policy" clean; \
		fi; \
	done

annotated-policy.wasm:
	@for policy in policies/*/; do \
		if [ -f "$$policy/Makefile" ]; then \
			echo "Building annotated-policy.wasm in $$policy"; \
			$(MAKE) -C "$$policy" annotated-policy.wasm; \
		fi; \
	done

test:
	$(call run-target,test)

lint:
	$(call run-target,lint)

e2e-tests:
	$(call run-target,e2e-tests)
