#!/usr/bin/env bash
# Builds one of ggez's examples into an Android APK, without Gradle.
#
#   ./build.sh [example] [--release] [--install] [--run]
#
# example defaults to 05_astroblasto. --install pushes the APK with adb, --run also
# launches it and follows its log output. See README.md for the SDK pieces this needs.
set -euo pipefail

EXAMPLE=05_astroblasto
PROFILE=debug
INSTALL=0
RUN=0
for arg in "$@"; do
    case "$arg" in
        --release) PROFILE=release ;;
        --install) INSTALL=1 ;;
        --run) INSTALL=1; RUN=1 ;;
        -*) echo "unknown flag $arg" >&2; exit 1 ;;
        *) EXAMPLE="$arg" ;;
    esac
done

: "${ANDROID_HOME:?set ANDROID_HOME to your Android SDK}"
NDK="${ANDROID_NDK_HOME:-$(ls -d "$ANDROID_HOME"/ndk/* | sort -V | tail -1)}"
BUILD_TOOLS="$(ls -d "$ANDROID_HOME"/build-tools/* | sort -V | tail -1)"
PLATFORM="$(ls -d "$ANDROID_HOME"/platforms/android-* | sort -V | tail -1)"
API=26
# arm64 for phones, x86_64 for the emulator.
TARGET="${ANDROID_TARGET:-aarch64-linux-android}"
case "$TARGET" in
    aarch64-linux-android) ABI=arm64-v8a ;;
    x86_64-linux-android) ABI=x86_64 ;;
    *) echo "unsupported ANDROID_TARGET $TARGET" >&2; exit 1 ;;
esac

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="$ROOT/target/android/$EXAMPLE"
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
TARGET_ENV="$(echo "$TARGET" | tr 'a-z-' 'A-Z_')"

export "CARGO_TARGET_${TARGET_ENV}_LINKER=$TOOLCHAIN/$TARGET$API-clang"
export "CC_${TARGET//-/_}=$TOOLCHAIN/$TARGET$API-clang"
export "CXX_${TARGET//-/_}=$TOOLCHAIN/$TARGET$API-clang++"
export "AR_${TARGET//-/_}=$TOOLCHAIN/llvm-ar"

CARGO_FLAGS=(-p ggez-android-examples --target "$TARGET")
[ "$PROFILE" = release ] && CARGO_FLAGS+=(--release)
GGEZ_EXAMPLE="$EXAMPLE" cargo build --manifest-path "$ROOT/Cargo.toml" "${CARGO_FLAGS[@]}"

PACKAGE="rs.ggez.examples.e$(echo "$EXAMPLE" | tr -c 'a-zA-Z0-9\n' '_')"
LABEL="ggez $EXAMPLE"

rm -rf "$OUT"
mkdir -p "$OUT/apk/lib/$ABI"
cp "$ROOT/target/$TARGET/$PROFILE/libggez_android_examples.so" "$OUT/apk/lib/$ABI/"
sed -e "s/PACKAGE/$PACKAGE/" -e "s/LABEL/$LABEL/" "$HERE/AndroidManifest.xml" > "$OUT/AndroidManifest.xml"

# The example's resources/ directory becomes the APK's assets/.
"$BUILD_TOOLS/aapt2" link -o "$OUT/unaligned.apk" \
    --manifest "$OUT/AndroidManifest.xml" \
    -I "$PLATFORM/android.jar" \
    -A "$ROOT/resources"
(cd "$OUT/apk" && zip -qr "$OUT/unaligned.apk" lib)
"$BUILD_TOOLS/zipalign" -f -p 4 "$OUT/unaligned.apk" "$OUT/aligned.apk"

KEYSTORE="${ANDROID_KEYSTORE:-$HOME/.android/debug.keystore}"
if [ ! -f "$KEYSTORE" ]; then
    mkdir -p "$(dirname "$KEYSTORE")"
    keytool -genkeypair -keystore "$KEYSTORE" -storepass android -alias androiddebugkey \
        -keypass android -keyalg RSA -validity 10000 -dname "CN=Android Debug,O=Android,C=US"
fi
APK="$OUT/$EXAMPLE.apk"
"$BUILD_TOOLS/apksigner" sign --ks "$KEYSTORE" --ks-pass pass:android --out "$APK" "$OUT/aligned.apk"
echo "Built $APK"

if [ "$INSTALL" = 1 ]; then
    adb install -r "$APK"
fi
if [ "$RUN" = 1 ]; then
    adb logcat -c
    adb shell am start -n "$PACKAGE/android.app.NativeActivity"
    adb logcat -v brief ggez:V RustStdoutStderr:V wgpu_hal:V '*:S'
fi
