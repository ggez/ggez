# Android examples

Android support is limited! This crate wraps an exampe in a `NativeActivity`.
`05_astroblasto` has touch controls, other examples with input won't work well.
For your own game, see the [Android guide](../../docs/BuildingForEveryPlatform.md#android).

## Build and run

The script currently requires Linux x86_64, Rust, a JDK (`java` and `keytool`),
`zip`, and an Android SDK with an SDK Platform (API 35 or later, with NDK), Build-Tools,
and `adb` (Platform-Tools). Set `ANDROID_HOME` to the SDK directory and put its `platform-tools`
on `PATH`. Optionally set `ANDROID_NDK_HOME`, otherwise the script selects the newest
available NDK, Build-Tools and SDK Platform.

From the repository root:

```sh
export ANDROID_HOME=/path/to/android-sdk
export PATH="$ANDROID_HOME/platform-tools:$PATH"
rustup target add aarch64-linux-android
./examples/android/build.sh 05_astroblasto
```

The APK is written to `target/android/05_astroblasto/05_astroblasto.apk`.
To install, launch and follow logcat output, connect a device with USB
debugging enabled (Android 8.0/API 26 or later), then run:

```sh
./examples/android/build.sh 05_astroblasto --run
```

Use `--install` to install without launching, or add `--release` for an optimized
build. Both profiles use a debug signing key, created at `~/.android/debug.keystore`
if absent. `--run` clears logcat before launching.

For an x86_64 emulator:

```sh
rustup target add x86_64-linux-android
ANDROID_TARGET=x86_64-linux-android ./examples/android/build.sh 05_astroblasto --run
```

The script packages the repository's `resources/` contents as APK assets. The
wrapper calls the selected example's `pub fn main()` after registering the Android
app with ggez, and routes logs and panics to logcat with `ggez` tag.
