.phony: help
help:
	@echo "Usage: make asb tag=username/asb:latest"

.phony: asb
asb:
	docker build -f asb.Dockerfile --cache-from $(tag) -t $(tag) ..
