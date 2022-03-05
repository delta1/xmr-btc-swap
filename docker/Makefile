SRC_DIR := $(shell cd .. && pwd)

.phony: asb
asb:
	docker run --rm -it -v $(SRC_DIR):/home/rust/src ekidd/rust-musl-builder cargo build --bin asb --release