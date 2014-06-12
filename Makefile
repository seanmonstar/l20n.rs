RUSTC ?= rustc

build:

test:
	$(RUSTC) --test src/libl20n/lib.rs && ./l20n
check: test
.PHONY: test
