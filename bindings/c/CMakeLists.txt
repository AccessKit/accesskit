cmake_minimum_required(VERSION 3.20)

project(accesskit_c)

include(accesskit-config.cmake)
include(FetchContent)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.3.5
)
FetchContent_MakeAvailable(Corrosion)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY "${ACCESSKIT_LIBRARIES_DIR}static")
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY "${ACCESSKIT_LIBRARIES_DIR}shared")
set(CMAKE_PDB_OUTPUT_DIRECTORY "${ACCESSKIT_LIBRARIES_DIR}shared")
corrosion_import_crate(MANIFEST_PATH Cargo.toml)

find_program(RUSTUP rustup)
find_program(CBINDGEN cbindgen)
find_program(CLANG_FORMAT clang-format)

add_custom_target(headers
    COMMAND ${RUSTUP} run nightly ${CBINDGEN} --crate accesskit_c --output "${ACCESSKIT_INCLUDE_DIR}/accesskit.hpp" "${CMAKE_SOURCE_DIR}"
    COMMAND ${CLANG_FORMAT} -i "${ACCESSKIT_INCLUDE_DIR}/accesskit.hpp"
    COMMAND ${CMAKE_COMMAND} -E rename "${ACCESSKIT_INCLUDE_DIR}/accesskit.hpp" "${ACCESSKIT_INCLUDE_DIR}/accesskit.h"
    BYPRODUCTS "${ACCESSKIT_INCLUDE_DIR}/accesskit.h"
)

add_dependencies(cargo-prebuild_accesskit headers)
