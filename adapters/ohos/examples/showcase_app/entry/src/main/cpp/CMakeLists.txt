cmake_minimum_required(VERSION 3.5.0)
project(accesskit_ohos_showcase)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(OHOS_ARCH STREQUAL "arm64-v8a")
    set(RUST_TARGET "aarch64-unknown-linux-ohos")
elseif(OHOS_ARCH STREQUAL "x86_64")
    set(RUST_TARGET "x86_64-unknown-linux-ohos")
elseif(OHOS_ARCH STREQUAL "armeabi-v7a")
    set(RUST_TARGET "armv7-unknown-linux-ohos")
else()
    message(FATAL_ERROR "Unsupported OHOS_ARCH: ${OHOS_ARCH}")
endif()

set(WORKSPACE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../../../../../..")
set(SHOWCASE_LIB "${WORKSPACE_ROOT}/target/${RUST_TARGET}/debug/examples/libshowcase.a")

add_library(showcase STATIC IMPORTED)
set_target_properties(showcase PROPERTIES IMPORTED_LOCATION "${SHOWCASE_LIB}")

add_library(entry SHARED napi_init.cpp)
target_link_libraries(entry PUBLIC libace_napi.z.so libace_ndk.z.so showcase)
