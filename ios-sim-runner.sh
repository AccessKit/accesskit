#!/bin/sh
set -e
set -o pipefail

command -v jq >/dev/null 2>&1 || { echo "Error: jq is required but not installed" >&2; exit 1; }
command -v xcrun >/dev/null 2>&1 || { echo "Error: xcrun (Xcode command line tools) is required but not installed" >&2; exit 1; }

EXECUTABLE="$1"
shift
ARGS="$@"
IDENTIFIER="dev.accesskit.TestRunner"
DISPLAY_NAME="TestRunner"
BUNDLE_NAME="${DISPLAY_NAME}.app"
EXECUTABLE_NAME=$(basename "$EXECUTABLE")
BUNDLE_PATH=$(dirname "$EXECUTABLE")/"${BUNDLE_NAME}"

# Minimal Info.plist for iOS sim app
PLIST="<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
<key>CFBundleIdentifier</key>
<string>${IDENTIFIER}</string>
<key>CFBundleDisplayName</key>
<string>${DISPLAY_NAME}</string>
<key>CFBundleName</key>
<string>${BUNDLE_NAME}</string>
<key>CFBundleExecutable</key>
<string>${EXECUTABLE_NAME}</string>
<key>CFBundleVersion</key>
<string>1.0</string>
<key>CFBundleShortVersionString</key>
<string>1.0</string>
<key>CFBundleDevelopmentRegion</key>
<string>en_US</string>
<key>UILaunchStoryboardName</key>
<string></string>
<key>LSRequiresIPhoneOS</key>
<true/>
</dict>
</plist>"

rm -rf "${BUNDLE_PATH}"
mkdir -p "${BUNDLE_PATH}"
echo "$PLIST" > "${BUNDLE_PATH}/Info.plist"
cp "$EXECUTABLE" "${BUNDLE_PATH}/"

# Helper functions for simulator management
ios_runtime() {
  runtime=$(xcrun simctl list -j runtimes ios | jq -r '.runtimes | sort_by(.identifier) | last.identifier')
  if [ -z "$runtime" ] || [ "$runtime" = "null" ]; then
    echo "Error: no iOS runtime found (is Xcode installed with iOS platform support?)" >&2
    exit 1
  fi
  echo "$runtime"
}

ios_device_id() {
  runtime=$(ios_runtime)
  device_id=$(xcrun simctl list -j devices | jq -r --arg rt "$runtime" '.devices[$rt][] | select(.name | contains("iPhone")) | select(.state == "Booted") | .udid' | head -1)
  if [ -z "$device_id" ]; then
    device_id=$(xcrun simctl list -j devices | jq -r --arg rt "$runtime" '.devices[$rt][] | select(.name | contains("iPhone")) | select(.state == "Shutdown") | .udid' | head -1)
    if [ -z "$device_id" ]; then
      echo "Error: no iPhone simulator found for runtime $runtime" >&2
      exit 1
    fi
    if ! xcrun simctl boot "$device_id" >&2; then
      echo "Error: failed to boot simulator $device_id" >&2
      exit 1
    fi
  fi
  if ! xcrun simctl bootstatus "$device_id" -b >&2; then
    echo "Error: simulator $device_id failed to reach booted state" >&2
    exit 1
  fi
  device_name=$(xcrun simctl list -j devices | jq -r --arg id "$device_id" '.devices | to_entries[] | .value[] | select(.udid == $id) | .name' | head -1)
  echo "Using simulator: $device_name ($device_id)" >&2
  echo "$device_id"
}

DEVICE_ID=$(ios_device_id)

xcrun simctl uninstall "$DEVICE_ID" "$IDENTIFIER" 2>/dev/null || true
xcrun simctl install "$DEVICE_ID" "$BUNDLE_PATH"

xcrun simctl spawn "$DEVICE_ID" "$BUNDLE_PATH/$EXECUTABLE_NAME" $ARGS
