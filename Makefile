ROOT_DIR := $(shell pwd)

EXAMPLE_DIR := ./poqr/example/
NTRU_DIR := ./poqr/ntru/

EX_OUT := $(ROOT_DIR)/example_prog

build:
	cargo build --manifest-path $(ROOT_DIR)/poqr/ntru/Cargo.toml
	cargo build --manifest-path $(ROOT_DIR)/poqr/example/Cargo.toml
	cp $(ROOT_DIR)/poqr/target/debug/example $(EX_OUT)
	

clean: 
	cargo clean --manifest-path $(ROOT_DIR)/poqr/ntru/Cargo.toml
	cargo clean --manifest-path $(ROOT_DIR)/poqr/example/Cargo.toml
	rm example_prog
