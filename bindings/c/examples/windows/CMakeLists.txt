cmake_minimum_required(VERSION 3.20)

project(windows_example)

include_directories(${ACCESSKIT_INCLUDE_DIR})

add_executable(hello_world hello_world.c)
if (${ACCESSKIT_IN_REPO})
    target_link_libraries(hello_world PUBLIC accesskit)
else()
    find_library(ACCESSKIT_LIB accesskit ${ACCESSKIT_LIBRARY_DIR})
    target_link_libraries(hello_world PUBLIC ${ACCESSKIT_LIB})
endif()

target_link_libraries(hello_world PUBLIC bcrypt uiautomationcore userenv ws2_32)
target_compile_definitions(hello_world PRIVATE -DUNICODE -D_UNICODE)
