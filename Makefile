RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUST_FLAGS ?=

.PHONY: all test

all: build
	$(RUSTC) $(RUST_FLAGS) --out-dir=target src/libl20n/lib.rs

build:
	test -d target || mkdir target

test: build
	$(RUSTC) --test src/libl20n/lib.rs -o target/test && ./target/test
check: test

docs: all
	rm -rf doc
	$(RUSTDOC) --test src/libl20n/lib.rs -L target
	$(RUSTDOC) src/libl20n/lib.rs



clean:
	rm -rf doc target/*
