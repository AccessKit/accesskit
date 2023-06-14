cmake_minimum_required(VERSION 3.20)

project(accesskit_c)

include(FetchContent)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.3.5
)
FetchContent_MakeAvailable(Corrosion)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
set(CMAKE_PDB_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
corrosion_import_crate(MANIFEST_PATH Cargo.toml)

find_program(RUSTUP rustup)
find_program(CBINDGEN cbindgen)
find_program(CLANG_FORMAT clang-format)

add_custom_target(headers
    COMMAND ${RUSTUP} run nightly ${CBINDGEN} --crate accesskit_c --output accesskit.hpp "${CMAKE_SOURCE_DIR}"
    COMMAND ${CLANG_FORMAT} -i accesskit.hpp
    COMMAND ${CMAKE_COMMAND} -E rename accesskit.hpp accesskit.h
    BYPRODUCTS accesskit.h
)

add_dependencies(cargo-prebuild_accesskit headers)

include("accesskit.cmake")

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/accesskit.h"
    DESTINATION "${ACCESSKIT_INCLUDE_DIR}"
)

install(FILES
    "$<TARGET_PROPERTY:accesskit-static,IMPORTED_LOCATION>"
    DESTINATION "${ACCESSKIT_LIBRARIES_DIR}/static"
)

install(FILES
    "$<TARGET_PROPERTY:accesskit-shared,IMPORTED_LOCATION>"
    "$<$<STREQUAL:${_accesskit_toolchain},msvc>:${CMAKE_CURRENT_BINARY_DIR}/accesskit.pdb>"
    DESTINATION "${ACCESSKIT_LIBRARIES_DIR}/shared"
)
install(FILES
    "$<$<NOT:$<STREQUAL:$<TARGET_PROPERTY:accesskit-shared,IMPORTED_IMPLIB>,>>:$<TARGET_PROPERTY:accesskit-shared,IMPORTED_IMPLIB>>"
    RENAME "accesskit.lib"
    DESTINATION "${ACCESSKIT_LIBRARIES_DIR}/shared"
)
