ROOT_DIR := $(shell pwd)

ONION_DIR := ./poqr/onion/
NTRU_DIR := ./poqr/ntru/
NETWORK_DIR := ./poqr/network/

VHOST_OUT := $(ROOT_DIR)/vhost
VROUTER_OUT := $(ROOT_DIR)/vrouter
ONION_OUT := $(ROOT_DIR)/poqr-run


build:
	cargo build --manifest-path $(ROOT_DIR)/poqr/Cargo.toml
	cp $(ROOT_DIR)/poqr/target/debug/vhost_main $(VHOST_OUT)
	cp $(ROOT_DIR)/poqr/target/debug/vrouter_main $(VROUTER_OUT)
	cp $(ROOT_DIR)/poqr/target/debug/onion $(ONION_OUT)

clean: 
	cargo clean --manifest-path $(ROOT_DIR)/poqr/Cargo.toml
	rm vhost
	rm vrouter
	rm poqr-run
