cmake_minimum_required(VERSION 3.20)

project(sdl_example)

find_package(SDL2 REQUIRED)
include_directories(${SDL2_INCLUDE_DIRS})
find_package(ACCESSKIT REQUIRED)
include_directories(windows_example ${ACCESSKIT_INCLUDE_DIR})

add_executable(hello_world hello_world.c)
target_link_libraries(hello_world PUBLIC ${SDL2_LIBRARIES})
target_link_libraries(hello_world PUBLIC accesskit)
target_compile_definitions(hello_world PRIVATE -DUNICODE -D_UNICODE)
