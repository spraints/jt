INSTALL_BIN_DIR := /home/spraints/.bin
SRCS = $(shell find src -name '*.rs')

.PHONY: install
install: $(INSTALL_BIN_DIR)/jt

$(INSTALL_BIN_DIR)/jt: target/release/jt
	cp $< $@

target/release/jt: $(SRCS)
	cargo build --release
