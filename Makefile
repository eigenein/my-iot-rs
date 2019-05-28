.PHONY: docker
docker:
	@docker build -t eigenein/my-iot-rs .

.PHONY: docs
docs:
	@cargo doc --document-private-items --no-deps
