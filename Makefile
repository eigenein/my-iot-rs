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
	@cargo test
	@cargo clippy --all-targets --all-features -- -D warnings
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

docker/build/%:
	@docker build -t "my-iot-rs/$*" -f Dockerfile . --target "$*"
	@docker run --rm -it -v "$(PWD):/my-iot-rs" "my-iot-rs/$*"
