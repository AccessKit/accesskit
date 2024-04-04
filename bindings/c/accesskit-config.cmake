include("${CMAKE_CURRENT_LIST_DIR}/accesskit.cmake")

add_library(accesskit INTERFACE)

add_library(accesskit-static STATIC IMPORTED GLOBAL)
find_library(_accesskit_static_lib accesskit "${ACCESSKIT_LIBRARIES_DIR}/static")
set_property(
    TARGET accesskit-static
    PROPERTY IMPORTED_LOCATION "${_accesskit_static_lib}"
)
set_property(
    TARGET accesskit-static
    PROPERTY INTERFACE_INCLUDE_DIRECTORIES "${ACCESSKIT_INCLUDE_DIR}"
)
if (_accesskit_os STREQUAL "macos")
    target_link_libraries(accesskit-static INTERFACE "-framework AppKit" "-framework Foundation" "-framework CoreFoundation" objc c++)
elseif (_accesskit_os STREQUAL "linux")
    target_link_libraries(accesskit-static INTERFACE -static-libgcc m)
elseif (_accesskit_os STREQUAL "windows")
    target_link_libraries(accesskit-static INTERFACE bcrypt ntdll uiautomationcore userenv ws2_32)
endif()

add_library(accesskit-shared SHARED IMPORTED GLOBAL)
if (_accesskit_os STREQUAL "windows")
    find_library(_accesskit_implib accesskit "${ACCESSKIT_LIBRARIES_DIR}/shared")
    set_property(
        TARGET accesskit-shared
        PROPERTY IMPORTED_IMPLIB "${_accesskit_implib}"
    )
endif()
if (_accesskit_os STREQUAL "macos")
    set(_accesskit_shared_lib "libaccesskit.dylib")
elseif (_accesskit_os STREQUAL "linux")
    set(_accesskit_shared_lib "libaccesskit.so")
elseif (_accesskit_os STREQUAL "windows")
    set(_accesskit_shared_lib "accesskit.dll")
endif()
set_property(
    TARGET accesskit-shared
    PROPERTY IMPORTED_LOCATION "${ACCESSKIT_LIBRARIES_DIR}/shared/${_accesskit_shared_lib}"
)
set_property(
    TARGET accesskit-shared
    PROPERTY INTERFACE_INCLUDE_DIRECTORIES "${ACCESSKIT_INCLUDE_DIR}"
)

if (BUILD_SHARED_LIBS)
    target_link_libraries(accesskit INTERFACE accesskit-shared)
else()
    target_link_libraries(accesskit INTERFACE accesskit-static)
endif()
