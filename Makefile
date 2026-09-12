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
	@node -e 'const fs = require("fs"); const p = "package-lock.json"; const lock = JSON.parse(fs.readFileSync(p, "utf8")); lock.version = "$(VERSION)"; lock.packages[""].version = "$(VERSION)"; fs.writeFileSync(p, JSON.stringify(lock, null, 2) + "\n");'
	@cd src-tauri && cargo update -p query2table --offline

release: bump-version
	@echo "Releasing v$(VERSION)..."
	git add VERSION package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
	git commit -m "chore(release): v$(VERSION)"
	git tag "v$(VERSION)"
	git push --atomic origin main "refs/tags/v$(VERSION)"
	@echo "Release v$(VERSION) pushed. CI will build and publish."
