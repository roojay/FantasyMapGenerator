cmake_minimum_required(VERSION 3.10)

project(map_generation)
set(CMAKE_BUILD_TYPE Release)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if (MSVC)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /O2 /W4 /D_CRT_SECURE_NO_WARNINGS")
else()
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O3 -Wall -Doff64_t=_off64_t")
endif()

include_directories(src)
file(GLOB SOURCES "src/*.cpp" "src/argtable3/*.c")

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_DEBUG ${CMAKE_BINARY_DIR})
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_RELEASE ${CMAKE_BINARY_DIR})

add_library(objects OBJECT ${SOURCES})
add_executable(map_generation $<TARGET_OBJECTS:objects>)

file(COPY "src/fontdata" DESTINATION ${CMAKE_BINARY_DIR})
file(COPY "src/citydata" DESTINATION ${CMAKE_BINARY_DIR})

set(RESOURCES_EXECUTABLE_DIRECTORY  ${CMAKE_BINARY_DIR})
set(RESOURCES_FONT_DATA_DIRECTORY   ${CMAKE_BINARY_DIR}/fontdata)
set(RESOURCES_CITY_DATA_DIRECTORY   ${CMAKE_BINARY_DIR}/citydata)
set(RESOURCES_FONT_DATA_RESOURCE    ${CMAKE_BINARY_DIR}/fontdata/fontdata.json)
set(RESOURCES_CITY_DATA_RESOURCE    ${CMAKE_BINARY_DIR}/citydata/countrycities.json)
configure_file(
  "${PROJECT_SOURCE_DIR}/src/resources.h.in"
  "${PROJECT_SOURCE_DIR}/src/resources.h"
)
