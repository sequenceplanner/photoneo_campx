cmake_minimum_required (VERSION 2.8.12)

set(CMAKE_CONFIGURATION_TYPES Release CACHE INTERNAL "" FORCE)
set(CMAKE_CXX_STANDARD 11)

macro(copyDlls)
    file(COPY ${PHOLOCALIZATION_OPTIMIZED_DLLs} DESTINATION "${ReleaseBinFolder}")
endmacro(copyDlls)

macro(copyFile Filename)
    file(COPY "${Filename}" DESTINATION "${CMAKE_BINARY_DIR}")
    file(COPY "${Filename}" DESTINATION "${ReleaseBinFolder}")
endmacro(copyFile)

project(Localize)

set(CMAKE_RELEASE_POSTFIX "_Release")

set(Files
    "${Localize_SOURCE_DIR}/Localize.cpp"
)

find_package(PhoLocalization REQUIRED CONFIG PATHS "$ENV{PHO_LOCALIZATION_PATH}")

add_executable(Localize
    ${Files}
)

target_link_libraries(Localize 
    ${PHOLOCALIZATION_LIBRARIES}
)

set_target_properties(Localize PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY_RELEASE bin
)

get_target_property(ReleaseBinFolder Localize RUNTIME_OUTPUT_DIRECTORY_RELEASE)

# copyFile("${PhoLocalizationConfig_PATH}/Examples/ExampleData/configurations/T-fitting.plcf")
# copyFile("${PhoLocalizationConfig_PATH}/Examples/ExampleData/scenes/T-fittings.ply")

if (WIN32)
    copyDlls()
    file(TO_CMAKE_PATH "$ENV{APPDATA}" APPDATA)
    if (EXISTS "${APPDATA}/PhotoneoLocalizationSDK/serial_key")
        copyFile("${APPDATA}/PhotoneoLocalizationSDK/serial_key")
    endif()
elseif (UNIX)
    if (EXISTS "$ENV{HOME}/.PhotoneoLocalizationSDK/serial_key")
        copyFile("$ENV{HOME}/.PhotoneoLocalizationSDK/serial_key")
    endif()
    add_compile_options(-std=c++1y)
endif ()

target_include_directories(Localize PUBLIC
    ${PHOLOCALIZATION_INCLUDE_DIRS}
)

set_target_properties(Localize
    PROPERTIES
    RELEASE_POSTFIX ${CMAKE_RELEASE_POSTFIX}
)
