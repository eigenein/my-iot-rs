# Please, follow the target naming conventions:
# https://www.gnu.org/prep/standards/html_node/Standard-Targets.html

EXECUTABLE_PATH := /usr/local/bin/my-iot

.PHONY: all
all:
	@RUSTFLAGS="-D warnings" cargo build --release

.PHONY: clean
clean:
	@cargo clean

.PHONY: check
check:
	@RUST_BACKTRACE=1 RUSTFLAGS="-D warnings" cargo test
	@cargo clippy --all-targets --all-features
	@cargo fmt --all -- --check

.PHONY: install
install:
	@cp target/release/my-iot $(EXECUTABLE_PATH)
	@setcap cap_net_raw+ep $(EXECUTABLE_PATH) || echo "Warning: install setcap to enable non-root ICMP"

.PHONY: uninstall
uninstall:
	@rm -f $(EXECUTABLE_PATH)

.PHONY: html docs
html docs:
	@cargo doc --document-private-items --no-deps
	@rsync -a --delete target/doc/ docs
	@echo '<html><head><meta http-equiv="refresh" content="0; url=my_iot"></head></html>' > docs/index.html
