.phony: help
help:
	@echo "Usage: make asb tag=username/asb:latest"

.phony: asb
asb:
	docker build -f asb.dockerfile -t $(tag) ..
