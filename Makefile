VERSION := $(shell cat VERSION | tr -d '[:space:]')

.PHONY: dev build check test release bump-version \
	android-init android-devices android-emulator android-dev android-apk android-install android-run android-logs android-keystore

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

# --- Android -------------------------------------------------------------------
# Needs ANDROID_HOME, NDK_HOME and JAVA_HOME (17) set, and adb on PATH.
# A phone needs USB debugging on; `make android-devices` should list it.

APP_ID := com.hightemp.query2table
ANDROID_DIR := src-tauri/gen/android
# Rust targets in the APK: phones (arm64, older armv7) and emulators (x86_64).
ANDROID_TARGETS ?= aarch64 armv7 x86_64
AVD ?= Medium_Phone_API_35
APK_OUT := $(ANDROID_DIR)/app/build/outputs/apk/universal/release/app-universal-release.apk
APK := dist/Query2Table_$(VERSION)_android.apk
KEYSTORE ?= $(HOME)/.android/query2table-release.jks
KEY_ALIAS ?= query2table

# Regenerate the Android project (only after changing its template; it is committed).
android-init:
	npm run tauri android init

android-devices:
	adb devices -l

# Starts an emulator in the background; pick another with AVD=<name> (`emulator -list-avds`).
android-emulator:
	nohup $(ANDROID_HOME)/emulator/emulator -avd $(AVD) -no-snapshot-load > /tmp/query2table-emulator.log 2>&1 &
	adb wait-for-device
	@echo "Emulator $(AVD) is starting; it is ready when the home screen shows."

# Live reload on the connected phone or emulator (the phone must reach this computer over the network).
android-dev:
	npm run tauri android dev

# Release APK for all targets, in dist/. Signed with your key when keystore.properties exists,
# otherwise with the debug key (installs for testing, cannot update a release-signed install).
android-apk:
	npm run tauri android build -- --apk $(addprefix --target ,$(ANDROID_TARGETS))
	@mkdir -p dist
	cp $(APK_OUT) $(APK)
	@echo "APK: $(APK)"

android-install:
	@test -f $(APK) || { echo "No $(APK); run make android-apk first"; exit 1; }
	adb install -r $(APK)

# Installs the APK on the connected device and opens the app.
android-run: android-install
	adb shell monkey -p $(APP_ID) -c android.intent.category.LAUNCHER 1 > /dev/null
	@echo "Started $(APP_ID); follow its logs with make android-logs"

# The app's log (Rust output and WebView console) while it runs.
android-logs:
	@pid=$$(adb shell pidof -s $(APP_ID) | tr -d '\r'); \
	if [ -z "$$pid" ]; then echo "$(APP_ID) is not running"; exit 1; fi; \
	adb logcat -v time --pid=$$pid

# Creates the release signing key (keep it safe: updates need the same key) and keystore.properties.
# Then add the printed values as repository secrets so releases are signed by CI.
android-keystore: SHELL := /bin/bash
android-keystore:
	@test ! -f $(KEYSTORE) || { echo "$(KEYSTORE) already exists"; exit 1; }
	@read -s -p "New key password (6+ characters): " pass; echo; \
	keytool -genkeypair -v -keystore $(KEYSTORE) -alias $(KEY_ALIAS) -keyalg RSA -keysize 4096 -validity 10000 \
		-storepass "$$pass" -keypass "$$pass" -dname "CN=Query2Table" && \
	printf 'storeFile=%s\nstorePassword=%s\nkeyAlias=%s\nkeyPassword=%s\n' "$(KEYSTORE)" "$$pass" "$(KEY_ALIAS)" "$$pass" \
		> $(ANDROID_DIR)/keystore.properties && \
	echo "Wrote $(ANDROID_DIR)/keystore.properties (git-ignored)." && \
	echo "For signed releases, add these GitHub secrets:" && \
	echo "  gh secret set ANDROID_KEYSTORE_BASE64 < <(base64 -w0 $(KEYSTORE))" && \
	echo "  gh secret set ANDROID_KEY_PASSWORD   # the password above" && \
	echo "  gh secret set ANDROID_KEY_ALIAS --body $(KEY_ALIAS)"

