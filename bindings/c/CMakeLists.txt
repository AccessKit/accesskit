cmake_minimum_required(VERSION 3.20)

project(accesskit_c)

SET(ACCESSKIT_IN_REPO ON)
if (NOT EXISTS(${CMAKE_SOURCE_DIR}/lib))
    SET(ACCESSKIT_IN_REPO OFF)
    if (WIN32)
        if (MINGW)
            if (CMAKE_SIZEOF_VOID_P MATCHES 8)
                SET(ACCESSKIT_LIBRARY_DIR ${CMAKE_SOURCE_DIR}/lib/windows/x86_64/mingw/static)
            else()
                SET(ACCESSKIT_LIBRARY_DIR ${CMAKE_SOURCE_DIR}/lib/windows/x86/mingw/static)
            endif()
        else()
            if (CMAKE_SIZEOF_VOID_P MATCHES 8)
                SET(ACCESSKIT_LIBRARY_DIR ${CMAKE_SOURCE_DIR}/lib/windows/x86_64/msvc/static)
            else()
                SET(ACCESSKIT_LIBRARY_DIR ${CMAKE_SOURCE_DIR}/lib/windows/x86/msvc/static)
            endif()
        endif()
    endif()
endif()
SET(ACCESSKIT_INCLUDE_DIR ${CMAKE_SOURCE_DIR}/include)

if (${ACCESSKIT_IN_REPO})
    include(ExternalProject)
    add_subdirectory(corrosion)
    corrosion_import_crate(MANIFEST_PATH Cargo.toml)

    add_custom_target(headers
        COMMAND rustup run nightly cbindgen --crate accesskit_c --output "${ACCESSKIT_INCLUDE_DIR}/accesskit.hpp" "${CMAKE_SOURCE_DIR}"
        COMMAND clang-format -i "${ACCESSKIT_INCLUDE_DIR}/accesskit.hpp"
        COMMAND ${CMAKE_COMMAND} -E rename "${ACCESSKIT_INCLUDE_DIR}/accesskit.hpp" "${ACCESSKIT_INCLUDE_DIR}/accesskit.h"
        DEPENDS "${ACCESSKIT_INCLUDE_DIR}/accesskit.h"
    )

    add_dependencies(cargo-prebuild_accesskit headers)
endif()

option(BUILD_WINDOWS_EXAMPLE "Build the Windows example" OFF)

if (BUILD_WINDOWS_EXAMPLE)
    add_subdirectory(examples/windows)
endif()
