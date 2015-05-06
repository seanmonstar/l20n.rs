RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUST_FLAGS ?=

.PHONY: all test

all: build
	$(RUSTC) $(RUST_FLAGS) --out-dir=target src/lib.rs

build:
	test -d target || mkdir target

test: build
	$(RUSTC) --test src/lib.rs -o target/test && ./target/test
check: test

docs: all
	rm -rf doc
	$(RUSTDOC) --test src/lib.rs -L target
	$(RUSTDOC) src/lib.rs



clean:
	rm -rf doc target/*
