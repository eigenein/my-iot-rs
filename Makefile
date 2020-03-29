# Please, follow the target naming conventions:
# https://www.gnu.org/prep/standards/html_node/Standard-Targets.html

EXECUTABLE_PATH := /usr/local/bin/my-iot

.PHONY: all
all:
	@RUSTFLAGS="-D warnings" cargo build --release

.PHONY: clean
clean:
	@cargo clean

.PHONY: check/clippy
check/clippy:
	@cargo clippy --workspace -- -D warnings

.PHONY: check/fmt
check/fmt:
	@cargo fmt -- --check

.PHONY: check/test
check/test:
	@cargo test --workspace

.PHONY: check
check: check/clippy check/fmt check/test

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
	@docker build -t "eigenein/my-iot-rs/$*" -f Dockerfile . --target "$*"
	@docker run --rm -v "$(PWD):/my-iot-rs" "eigenein/my-iot-rs/$*"
