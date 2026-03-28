VERSION := $(shell cat VERSION | tr -d '[:space:]')

.PHONY: dev build check test release bump-version

dev:
	npm run tauri dev

build:
	npm run tauri build

check:
	npm run check
	cd src-tauri && cargo check

test:
	npm test
	cd src-tauri && cargo test

bump-version:
	@echo "Bumping version to $(VERSION) in all files..."
	@sed -i 's/"version": "[^"]*"/"version": "$(VERSION)"/' package.json
	@sed -i 's/"version": "[^"]*"/"version": "$(VERSION)"/' src-tauri/tauri.conf.json
	@sed -i 's/^version = "[^"]*"/version = "$(VERSION)"/' src-tauri/Cargo.toml

release: bump-version
	@echo "Releasing v$(VERSION)..."
	git add -A
	git commit -m "release: v$(VERSION)" --allow-empty
	git tag -f "v$(VERSION)"
	git push -f origin main
	git push -f origin "v$(VERSION)"
	@echo "Release v$(VERSION) pushed. CI will build and publish."
