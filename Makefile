ROOT_DIR := $(shell pwd)

ONION_DIR := ./poqr/onion/
NTRU_DIR := ./poqr/ntru

clean: 
	cargo clean --manifest-path $(ONION_DIR)/Cargo.toml
