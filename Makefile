# Please, follow the target naming conventions:
# https://www.gnu.org/prep/standards/html_node/Standard-Targets.html

EXECUTABLE_PATH := /usr/local/bin/my-iot

.PHONY: all
all:
	@cargo build --release

.PHONY: clean
clean:
	@cargo clean

.PHONY: check
check:
	@cargo fmt --all -- --check
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo test

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
	@rsync -a target/doc/ docs
