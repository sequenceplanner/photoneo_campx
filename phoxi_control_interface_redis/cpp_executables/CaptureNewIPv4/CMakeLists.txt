# CMakeLists files in this project can
# refer to the root source directory of the project as ${CaptureNewIPv4_SOURCE_DIR} and
# to the root binary directory of the project as ${CaptureNewIPv4_BINARY_DIR}.
cmake_minimum_required (VERSION 3.10)

if(POLICY CMP0054)
    cmake_policy(SET CMP0054 NEW)
endif()

project (CaptureNewIPv4)

set(CMAKE_RELEASE_POSTFIX "_Release")
set(CMAKE_DEBUG_POSTFIX "_Debug")

if (UNIX AND NOT APPLE)
    add_compile_options(-std=c++1y)
    add_compile_options(-pthread)
endif(UNIX AND NOT APPLE)

set(Files
    ${CaptureNewIPv4_SOURCE_DIR}/CaptureNewIPv4.cpp
)

add_executable (CaptureNewIPv4 
    ${Files}
)

find_package(PhoXi REQUIRED CONFIG PATHS "$ENV{PHOXI_CONTROL_PATH}")

if (UNIX AND NOT APPLE)
    target_link_libraries(CaptureNewIPv4
        ${PHOXI_LIBRARY}
        rt
    )
else (UNIX AND NOT APPLE)
    target_link_libraries(CaptureNewIPv4
        ${PHOXI_LIBRARY}
    )
endif(UNIX AND NOT APPLE)

get_target_property(ReleaseBinFolder CaptureNewIPv4 RUNTIME_OUTPUT_DIRECTORY_RELEASE)
get_target_property(DebugBinFolder CaptureNewIPv4 RUNTIME_OUTPUT_DIRECTORY_DEBUG)

if(MSVC)
    add_custom_command(TARGET CaptureNewIPv4 POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different  # which executes "cmake - E copy_if_different..."
            "${PHOXI_DLL_DEBUG}"
            $<TARGET_FILE_DIR:CaptureNewIPv4>
    )

    add_custom_command(TARGET CaptureNewIPv4 POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different  # which executes "cmake - E copy_if_different..."
            "${PHOXI_DLL_RELEASE}"
            $<TARGET_FILE_DIR:CaptureNewIPv4>
    )
endif(MSVC)

target_include_directories (CaptureNewIPv4 PUBLIC ${PHOXI_INCLUDE_DIRS})

set_target_properties(CaptureNewIPv4
    PROPERTIES
    #for executables, inheritance of variables CMAKE_DEBUG_POSTFIX... does not work
    DEBUG_POSTFIX ${CMAKE_DEBUG_POSTFIX}
    RELEASE_POSTFIX ${CMAKE_RELEASE_POSTFIX}
)