#!/usr/bin/env bash
set -eux
: "${1:?example name required}"
: "${SRCROOT:?}" "${DERIVED_FILE_DIR:?}" "${TARGET_BUILD_DIR:?}" "${EXECUTABLE_PATH:?}" "${ARCHS:?}" "${PLATFORM_NAME:?}"
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH:$HOME/.cargo/bin"

if [[ "$CONFIGURATION" != "Debug" ]]; then
    CARGO_PROFILE=release
    cargo_args=(--release)
else
    CARGO_PROFILE=debug
    cargo_args=()
fi

# Make Cargo output cache files in Xcode's directories
export CARGO_TARGET_DIR="$DERIVED_FILE_DIR/cargo"

case "$PLATFORM_NAME" in
    iphoneos)         CARGO_OS=ios;      BUILD_KIND=device ;;
    iphonesimulator)  CARGO_OS=ios;      BUILD_KIND=simulator ;;
    macosx)
        if [[ "${IS_MACCATALYST:-NO}" != "YES" ]]; then
            echo "non-Catalyst macOS builds are not supported" >&2
            exit 1
        fi
        CARGO_OS=ios
        BUILD_KIND=catalyst
        ;;
    *)
        echo "unsupported platform: $PLATFORM_NAME" >&2
        exit 1
        ;;
esac

cd "$SRCROOT/../.."

executables=()
for arch in $ARCHS; do
    case "$arch" in
        arm64)  RUST_ARCH=aarch64 ;;
        x86_64) RUST_ARCH=x86_64 ;;
        *)
            echo "unsupported arch: $arch" >&2
            exit 1
            ;;
    esac

    case "$BUILD_KIND" in
        device)
            if [[ "$RUST_ARCH" = "x86_64" ]]; then
                echo "x86_64 is not valid for device builds" >&2
                exit 1
            fi
            CARGO_TARGET="${RUST_ARCH}-apple-${CARGO_OS}"
            ;;
        simulator)
            if [[ "$RUST_ARCH" = "x86_64" ]]; then
                # Rust names the x86_64 simulator target without a -sim suffix,
                # but clang still needs the -simulator triple.
                CARGO_TARGET="${RUST_ARCH}-apple-${CARGO_OS}"
                export "CFLAGS_${RUST_ARCH}_apple_${CARGO_OS}=-target ${RUST_ARCH}-apple-${CARGO_OS}-simulator"
            else
                CARGO_TARGET="${RUST_ARCH}-apple-${CARGO_OS}-sim"
            fi
            ;;
        catalyst)
            CARGO_TARGET="${RUST_ARCH}-apple-ios-macabi"
            ;;
    esac

    cargo build ${cargo_args[@]+"${cargo_args[@]}"} \
        --target "$CARGO_TARGET" \
        --example "$1" \
        --no-default-features --features rwh_06

    executables+=("$DERIVED_FILE_DIR/cargo/$CARGO_TARGET/$CARGO_PROFILE/examples/$1")
done

lipo -create -output "$TARGET_BUILD_DIR/$EXECUTABLE_PATH" "${executables[@]}"

