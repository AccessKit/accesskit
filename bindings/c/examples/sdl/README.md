# AccessKit cross-platform SDL example

This example demonstrates how to make use of the C bindings to create cross-platform applications.

## Building

The process will vary based on your operating system.

### Windows:

First download an SDL2 development package from the project's [GitHub release page](https://github.com/libsdl-org/SDL/releases) (SDL2-devel-2.x.y-VC.zip for MSVC) and extract it.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DSDL2_DIR="<PATH_TO_SDL2_PACKAGE>/cmake"
cmake --build build --config Release
```

You will then need to copy `SDL2.dll` into the `build/Release` folder.

### Linux

Make sure to install SDL2 and its development package.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

### macOS

First download an SDL2 package from the project's [GitHub release page](https://github.com/libsdl-org/SDL/releases) (SDL2-2.x.y.dmg) and copy `SDL2.framework` to `/Library/Frameworks`.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DCMAKE_BUILD_TYPE=Release
cmake --build build
```
