# Please, follow the target naming conventions:
# https://www.gnu.org/prep/standards/html_node/Standard-Targets.html

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

.PHONY: html docs
html docs:
	@mdbook build
	@touch docs

docker/build/%:
	@docker build -t "eigenein/my-iot-rs/$*" -f Dockerfile . --target "$*"
	@docker run --rm -v "$(PWD):/my-iot-rs" "eigenein/my-iot-rs/$*"

.PHONY: src/statics
src/statics:
	@curl 'https://cdnjs.cloudflare.com/ajax/libs/bulma/0.9.0/css/bulma.min.css' --output 'src/statics/bulma.min.css'
	@curl 'https://unpkg.com/bulma-prefers-dark@0.1.0-beta.0/css/bulma-prefers-dark.css' --output 'src/statics/bulma-prefers-dark.css'
	@curl --location 'https://github.com/chartjs/Chart.js/releases/download/v2.9.3/Chart.bundle.min.js' --output 'src/statics/Chart.bundle.min.js'
	@curl 'https://use.fontawesome.com/releases/v5.13.1/fontawesome-free-5.13.1-web.zip' --output src/statics/fontawesome.zip
	@unzip -u src/statics/fontawesome.zip -d src/statics/
	@rm src/statics/fontawesome.zip

.PHONY: tag
tag:
	@$(eval VERSION := $(shell cargo run -- --version))
	@git tag -a $(VERSION) -m $(VERSION)

.PHONY: tag/publish
tag/publish: tag
	@git push origin $(shell cargo run -- --version)

.PHONY: publish
publish: tag/publish
	@cargo publish
