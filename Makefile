# Please, follow the target naming conventions:
# https://www.gnu.org/prep/standards/html_node/Standard-Targets.html

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

install: target/release/my-iot
	@cp $< /usr/local/bin/my-iot

.PHONY: html
html:
	@cargo doc --document-private-items --no-deps
	@cp -R target/doc docs

.PHONY: setup
setup:
	@rustup component add rustfmt clippy
