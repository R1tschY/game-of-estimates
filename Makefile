.PHONY: all

all: frontend-release backend-release

frontend-release:
	cd frontend && npm install && npm run build

backend-release:
	cargo build --release
