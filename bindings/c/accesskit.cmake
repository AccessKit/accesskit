set(ACCESSKIT_INCLUDE_DIR "${CMAKE_CURRENT_LIST_DIR}/include")
set(_accesskit_toolchain "")

if (APPLE)
    set(_accesskit_os "macos")
    if (CMAKE_OSX_ARCHITECTURES MATCHES "(ARM64|arm64|aarch64)")
        set(_accesskit_arch "arm64")
    elseif (CMAKE_OSX_ARCHITECTURES MATCHES "(AMD64|amd64|x86_64)")
        set(_accesskit_arch "x86_64")
    endif()
elseif (UNIX)
    set(_accesskit_os "linux")
elseif (WIN32)
    set(_accesskit_os "windows")
    if (MINGW)
        set(_accesskit_toolchain "mingw")
    else()
        set(_accesskit_toolchain "msvc")
    endif()

    if (CMAKE_VS_PLATFORM_NAME)
        string(TOLOWER "${CMAKE_VS_PLATFORM_NAME}" LOWER_VS_PLATFORM_NAME)
        if ("${LOWER_VS_PLATFORM_NAME}" STREQUAL "win32")
            set(_accesskit_arch x86)
        elseif("${LOWER_VS_PLATFORM_NAME}" STREQUAL "x64")
            set(_accesskit_arch x86_64)
        elseif ("${LOWER_VS_PLATFORM_NAME}" STREQUAL "arm64")
            set(_accesskit_arch "arm64")
        endif()
    endif()
endif()

if (NOT _accesskit_arch)
   if (CMAKE_SYSTEM_PROCESSOR MATCHES "^(AMD64|amd64|x86_64)$")
        set(_accesskit_arch x86_64)
    elseif (CMAKE_SYSTEM_PROCESSOR MATCHES "^(ARM64|arm64|aarch64)$")
        set(_accesskit_arch arm64)
    elseif (CMAKE_SYSTEM_PROCESSOR MATCHES "^(X86|x86|i686)$")
        set(_accesskit_arch x86)
    endif()
endif()

set(ACCESSKIT_LIBRARIES_DIR "${CMAKE_CURRENT_LIST_DIR}/lib/${_accesskit_os}/${_accesskit_arch}")
if (_accesskit_toolchain)
    string(APPEND ACCESSKIT_LIBRARIES_DIR "/${_accesskit_toolchain}")
endif()
