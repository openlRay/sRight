#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
DEVELOPER_DIR=${DEVELOPER_DIR:-/Applications/Xcode.app/Contents/Developer}
export DEVELOPER_DIR
DEVELOPMENT_TEAM=${DEVELOPMENT_TEAM:-98PZ67DD35}
SIGN_IDENTITY=${SIGN_IDENTITY:-Apple Development: smfvacant@gmail.com (2NTYM5DJBW)}

cd "$ROOT_DIR"

pnpm --filter @sright/desktop tauri build
cargo build -p sright-cli --release

xcodebuild \
  -project native/macos/sRightNative.xcodeproj \
  -target SRightFinderSync \
  -configuration Release \
  SYMROOT="$ROOT_DIR/build/xcode/Build/Products" \
  OBJROOT="$ROOT_DIR/build/xcode/Build/Intermediates.noindex" \
  CODE_SIGN_IDENTITY="Apple Development" \
  CODE_SIGN_STYLE=Automatic \
  CODE_SIGN_ALLOW_ENTITLEMENTS_MODIFICATION=YES \
  DEVELOPMENT_TEAM="$DEVELOPMENT_TEAM" \
  -allowProvisioningUpdates \
  build

APP_PATH="$ROOT_DIR/target/release/bundle/macos/sRight.app"
APPEX_PATH="$ROOT_DIR/build/xcode/Build/Products/Release/SRightFinderSync.appex"
CLI_PATH="$ROOT_DIR/target/release/sright-cli"
PKG_ROOT=$(mktemp -d "$ROOT_DIR/dist/pkg-root.XXXXXX")
COMPONENTS_PLIST="$ROOT_DIR/dist/components.plist"
PKG_PATH="$ROOT_DIR/dist/sRight-0.1.5-local.pkg"

rm -rf "$APP_PATH/Contents/PlugIns"
mkdir -p "$APP_PATH/Contents/PlugIns"
ditto "$APPEX_PATH" "$APP_PATH/Contents/PlugIns/SRightFinderSync.appex"
cp "$CLI_PATH" "$APP_PATH/Contents/MacOS/sright-cli"
chmod 755 "$APP_PATH/Contents/MacOS/sright-cli"

codesign --force --sign "$SIGN_IDENTITY" "$APP_PATH/Contents/MacOS/sright-cli"
codesign --force --sign "$SIGN_IDENTITY" --entitlements "$ROOT_DIR/native/macos/FinderSyncExtension/SRightFinderSync.entitlements" "$APP_PATH/Contents/PlugIns/SRightFinderSync.appex"
codesign --force --sign "$SIGN_IDENTITY" "$APP_PATH"

rm -f "$PKG_PATH" "$COMPONENTS_PLIST"
mkdir -p "$PKG_ROOT/Applications" "$PKG_ROOT/usr/local/bin"
ditto "$APP_PATH" "$PKG_ROOT/Applications/sRight.app"
cp "$CLI_PATH" "$PKG_ROOT/usr/local/bin/sright-cli"
chmod 755 "$PKG_ROOT/usr/local/bin/sright-cli"
codesign --force --sign "$SIGN_IDENTITY" "$PKG_ROOT/usr/local/bin/sright-cli"

pkgbuild --analyze --root "$PKG_ROOT" "$COMPONENTS_PLIST"
/usr/libexec/PlistBuddy -c "Set :0:BundleIsRelocatable false" "$COMPONENTS_PLIST"

pkgbuild \
  --root "$PKG_ROOT" \
  --identifier dev.sright.local \
  --version 0.1.5 \
  --install-location / \
  --component-plist "$COMPONENTS_PLIST" \
  "$PKG_PATH"

pkgutil --payload-files "$PKG_PATH" | grep -E 'SRightFinderSync\.appex|sRight\.app|sright-cli' >/dev/null
shasum -a 256 "$PKG_PATH"
