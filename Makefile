ROOT_DIR := $(shell pwd)

ONION_DIR := ./poqr/onion/
NTRU_DIR := ./poqr/ntru/
NETWORK_DIR := ./poqr/network/

clean: 
	cargo clean --manifest-path $(ONION_DIR)/Cargo.toml
	cargo clean --manifest-path $(NTRU_DIR)/Cargo.toml
	cargo clean --manifest-path $(NETWORK_DIR)/Cargo.toml
