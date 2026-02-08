

IGNORE = -i temp/ -i target/ -i target_dist/


.PHONY: dev
dev:
	cargo watch $(IGNORE) --clear -x 'run --example base'