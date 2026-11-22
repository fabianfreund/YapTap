#!/usr/bin/env bash
# Build, sign (Developer ID: Digital Vibes GmbH), notarize and publish the macOS build.
# Windows and Linux are built (unsigned) by .github/workflows/release.yml when the tag is pushed.
#
#   pnpm release:mac              # build + notarize + upload to the GitHub release of this version
#   SKIP_UPLOAD=1 pnpm release:mac  # local signed + notarized build only
#   NOTARIZE=0 SKIP_UPLOAD=1 ...    # signed only, quick local test (Gatekeeper rejects it)
#
# Needs: the Developer ID certificate in the login keychain and the notarytool keychain
# profile "splatcraft" (xcrun notarytool store-credentials splatcraft --apple-id <id> --team-id 8HFX6YPXXN).
# v0.1 ships Apple Silicon only; Intel (x86_64, CPU backend) can be added as a second target.
set -euo pipefail
cd "$(dirname "$0")/.."

REPO="fabianfreund/YapTap"
VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version")
TAG="v$VERSION"
PROFILE="${APPLE_KEYCHAIN_PROFILE:-splatcraft}"
export APPLE_SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:-Developer ID Application: Digital Vibes GmbH (8HFX6YPXXN)}"
unset APPLE_ID APPLE_PASSWORD APPLE_API_KEY APPLE_API_ISSUER # we notarize below with the keychain profile

if [[ -z "${SKIP_UPLOAD:-}" && -n "$(git status --porcelain)" ]]; then
  echo "Working tree is not clean. Commit first (or use SKIP_UPLOAD=1)." >&2; exit 1
fi
for f in package.json src-tauri/Cargo.toml; do
  grep -q "\"version\": \"$VERSION\"\|^version = \"$VERSION\"" "$f" || { echo "$f is not at $VERSION" >&2; exit 1; }
done

echo "▸ YapTap $TAG (Apple Silicon)"
pnpm install --frozen-lockfile
pnpm check
pnpm tauri build --target aarch64-apple-darwin --bundles app,dmg

BUNDLE=src-tauri/target/aarch64-apple-darwin/release/bundle
APP="$BUNDLE/macos/YapTap.app"
DMG_SRC=$(ls "$BUNDLE"/dmg/*.dmg | head -1)
OUT=$(mktemp -d)
DMG="$OUT/YapTap_${VERSION}_aarch64.dmg"

notarize() { # file
  echo "▸ Notarizing $(basename "$1")"
  xcrun notarytool submit "$1" --keychain-profile "$PROFILE" --wait | tee "$OUT/notary.log"
  grep -q "status: Accepted" "$OUT/notary.log" || { echo "Notarization failed" >&2; exit 1; }
}

codesign --verify --deep --strict "$APP"
cp "$DMG_SRC" "$DMG"
if [[ "${NOTARIZE:-1}" != "0" ]]; then
  ditto -c -k --keepParent "$APP" "$OUT/app.zip"
  notarize "$OUT/app.zip"
  xcrun stapler staple "$APP"
  notarize "$DMG"
  xcrun stapler staple "$DMG"
  spctl -a -t exec -vv "$APP"
fi
ls -lh "$DMG"
[[ -n "${SKIP_UPLOAD:-}" ]] && { echo "Artifacts in $OUT"; exit 0; }

if ! git rev-parse "$TAG" >/dev/null 2>&1; then git tag -a "$TAG" -m "YapTap $TAG"; fi
git push origin "$TAG"
echo "▸ Waiting for CI to create the draft release"
for _ in $(seq 1 60); do gh release view "$TAG" -R "$REPO" >/dev/null 2>&1 && break; sleep 10; done
gh release view "$TAG" -R "$REPO" >/dev/null 2>&1 || gh release create "$TAG" -R "$REPO" --title "YapTap $TAG" --draft --verify-tag
gh release upload "$TAG" -R "$REPO" "$DMG" --clobber
echo "▸ Uploaded. Publish the draft once the Windows/Linux builds are attached:"
echo "  gh release edit $TAG -R $REPO --draft=false --latest"
