cmake_minimum_required(VERSION 3.25)

project(assdraw VERSION 3.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(CTest)
option(ASSDRAW_BUILD_APP "Build the graphical editor" ON)
find_package(Python3 REQUIRED COMPONENTS Interpreter)

if(BUILD_TESTING)
    add_executable(assdraw-core-tests
        tests/style_history_tests.cpp
        src/shape_style.cpp
        src/styled_ass.cpp
    )
    target_include_directories(assdraw-core-tests PRIVATE src)
    if(MSVC)
        target_compile_options(assdraw-core-tests PRIVATE /UNDEBUG)
    else()
        target_compile_options(assdraw-core-tests PRIVATE -UNDEBUG)
    endif()
    add_test(NAME assdraw-core-tests COMMAND assdraw-core-tests)
    find_package(Python3 COMPONENTS Interpreter)
    if(Python3_Interpreter_FOUND)
        add_test(NAME assdraw-pan-drag-tests
            COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tests/pan_drag_wiring_tests.py)
        add_test(NAME assdraw-translations-check
            COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tools/translations.py --check)
        add_test(NAME assdraw-manual-check
            COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tools/check_manual.py)
    endif()
endif()

if(NOT ASSDRAW_BUILD_APP)
    return()
endif()

if(WIN32)
    enable_language(RC)
endif()

if(MSVC)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>DLL")
endif()

set(ASSDRAW_AGG_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/vendor/agg/agg-src" CACHE PATH "AGG source directory")
if(NOT EXISTS "${ASSDRAW_AGG_ROOT}/include/agg_basics.h")
    include(FetchContent)
    FetchContent_Declare(agg
        GIT_REPOSITORY https://github.com/ghaerr/agg-2.6.git
        GIT_TAG c4f36b4432142f22c0bf82c6fbdb41567a236be2
        SOURCE_SUBDIR __assdraw_no_upstream_cmake_project
    )
    FetchContent_MakeAvailable(agg)
    set(ASSDRAW_AGG_ROOT "${agg_SOURCE_DIR}/agg-src")
endif()

set(ASSDRAW_AGG_SOURCES
    "${ASSDRAW_AGG_ROOT}/src/agg_arc.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_arrowhead.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_bezier_arc.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_bspline.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_color_rgba.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_curves.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_embedded_raster_fonts.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_gsv_text.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_image_filters.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_line_aa_basics.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_line_profile_aa.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_rounded_rect.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_sqrt_tables.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_trans_affine.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_trans_double_path.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_trans_single_path.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_trans_warp_magnifier.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vcgen_bspline.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vcgen_contour.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vcgen_dash.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vcgen_markers_term.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vcgen_smooth_poly1.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vcgen_stroke.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vpgen_clip_polygon.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vpgen_clip_polyline.cpp"
    "${ASSDRAW_AGG_ROOT}/src/agg_vpgen_segmentator.cpp"
)

add_library(assdraw_agg STATIC ${ASSDRAW_AGG_SOURCES})
target_include_directories(assdraw_agg PUBLIC "${ASSDRAW_AGG_ROOT}/include")
target_compile_definitions(assdraw_agg PRIVATE _CRT_SECURE_NO_WARNINGS NOMINMAX)

option(ASSDRAW_BUNDLED_WX "Build the pinned wxWidgets with native runtime dark mode" ON)
if(ASSDRAW_BUNDLED_WX)
    include(FetchContent)
    set(wxBUILD_SHARED OFF CACHE BOOL "" FORCE)
    set(wxBUILD_USE_STATIC_RUNTIME OFF CACHE BOOL "" FORCE)
    set(wxBUILD_INSTALL OFF CACHE BOOL "" FORCE)
    set(wxBUILD_SAMPLES OFF CACHE STRING "" FORCE)
    set(wxBUILD_TESTS OFF CACHE STRING "" FORCE)
    set(wxBUILD_DEMOS OFF CACHE BOOL "" FORCE)
    set(wxBUILD_LOCALES OFF CACHE BOOL "" FORCE)
    foreach(feature WEBVIEW MEDIACTRL OPENGL STC RICHTEXT XRC LIBNOTIFY LIBSDL SOUND)
        set(wxUSE_${feature} OFF CACHE BOOL "" FORCE)
    endforeach()
    foreach(library ZLIB EXPAT LIBJPEG LIBPNG LIBTIFF REGEX NANOSVG)
        set(wxUSE_${library} builtin CACHE STRING "" FORCE)
    endforeach()
    set(wxUSE_LIBWEBP OFF CACHE STRING "" FORCE)
    FetchContent_Declare(wxwidgets
        URL https://github.com/wxWidgets/wxWidgets/releases/download/v3.3.4/wxWidgets-3.3.4.tar.bz2
        URL_HASH SHA256=815c16cafa4fcbfcf25c5c0a3418309c62257b8a3b6f91d85a1367a134f98c98
        DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
    FetchContent_MakeAvailable(wxwidgets)
    set(ASSDRAW_WX_LIBRARIES wx::base wx::core wx::aui wx::html wx::propgrid)
    install(FILES "${wxwidgets_SOURCE_DIR}/docs/licence.txt" DESTINATION licenses RENAME wxWidgets.txt)
    install(FILES "${wxwidgets_SOURCE_DIR}/src/jpeg/README" DESTINATION licenses RENAME jpeg.txt)
    foreach(thirdparty_license src/expat/COPYING src/zlib/LICENSE src/tiff/LICENSE.md
            src/png/LICENSE 3rdparty/pcre/LICENCE.md 3rdparty/nanosvg/LICENSE.txt)
        get_filename_component(license_directory "${thirdparty_license}" DIRECTORY)
        get_filename_component(license_name "${license_directory}" NAME)
        install(FILES "${wxwidgets_SOURCE_DIR}/${thirdparty_license}"
            DESTINATION licenses RENAME "${license_name}.txt")
    endforeach()
else()
    set(wxWidgets_USE_STATIC OFF)
    find_package(wxWidgets 3.3.4 REQUIRED COMPONENTS base core adv aui html propgrid)
    include(${wxWidgets_USE_FILE})
    set(ASSDRAW_WX_LIBRARIES ${wxWidgets_LIBRARIES})
endif()

if(UNIX AND NOT APPLE)
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(ASSDRAW_GTK REQUIRED IMPORTED_TARGET gtk+-3.0)
endif()

file(GLOB_RECURSE ASSDRAW_ICON_SVGS CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/vendor/fontawesome/svgs/*.svg")
set(ASSDRAW_ICON_HEADER "${CMAKE_CURRENT_BINARY_DIR}/generated/assdraw_icons_data.hpp")
add_custom_command(OUTPUT "${ASSDRAW_ICON_HEADER}"
    COMMAND ${Python3_EXECUTABLE} "${CMAKE_CURRENT_SOURCE_DIR}/tools/embed_icons.py"
        "${CMAKE_CURRENT_SOURCE_DIR}/vendor/fontawesome" "${ASSDRAW_ICON_HEADER}"
    DEPENDS tools/embed_icons.py ${ASSDRAW_ICON_SVGS} VERBATIM)
add_custom_target(assdraw-icons DEPENDS "${ASSDRAW_ICON_HEADER}")

set(ASSDRAW_BITMAP_HEADER "${CMAKE_CURRENT_BINARY_DIR}/generated/assdraw_bitmap_data.hpp")
add_custom_command(OUTPUT "${ASSDRAW_BITMAP_HEADER}"
    COMMAND ${Python3_EXECUTABLE} "${CMAKE_CURRENT_SOURCE_DIR}/tools/embed_bitmap.py"
        "${CMAKE_CURRENT_SOURCE_DIR}/src/bitmaps/assdraw3.bmp" "${ASSDRAW_BITMAP_HEADER}"
    DEPENDS tools/embed_bitmap.py src/bitmaps/assdraw3.bmp VERBATIM)
add_custom_target(assdraw-bitmaps DEPENDS "${ASSDRAW_BITMAP_HEADER}")

set(ASSDRAW_SOURCES
    src/assdraw.cpp
    src/assdraw_settings.cpp
    src/canvas.cpp
    src/canvas_mouse.cpp
    src/cmd.cpp
    src/dlgctrl.cpp
    src/engine.cpp
    src/library.cpp
    src/settings.cpp
    src/shape_style.cpp
    src/styled_ass.cpp
    src/localization.cpp
    src/theme.cpp
    src/frame_theme.cpp
    src/icons.cpp
    src/wxAGG/AGGWindow.cpp
    src/xpm/res.cpp
)

set(ASSDRAW_HEADERS
    src/_common.hpp
    src/agghelper.hpp
    src/assdraw.hpp
    src/canvas.hpp
    src/canvas_mouse.hpp
    src/cmd.hpp
    src/dlgctrl.hpp
    src/engine.hpp
    src/enums.hpp
    src/include_once.hpp
    src/library.hpp
    src/resource.h
    src/settings.hpp
    src/drawing_history.hpp
    src/shape_style.hpp
    src/styled_ass.hpp
    src/localization.hpp
    src/theme.hpp
    src/icons.hpp
    src/wxAGG/AGGWindow.h
    src/wxAGG/PixelFormatConvertor.h
    src/xpm/res.h
)

set(ASSDRAW_RESOURCES)
if(WIN32)
    list(APPEND ASSDRAW_RESOURCES src/assdraw.rc)
    list(APPEND ASSDRAW_RESOURCES src/dpi_aware.manifest)
endif()

add_executable(
    assdraw
    WIN32
    ${ASSDRAW_SOURCES}
    ${ASSDRAW_HEADERS}
    ${ASSDRAW_RESOURCES}
)

target_include_directories(
    assdraw
    PRIVATE
        src
        src/wxAGG
        src/xpm
)

target_compile_definitions(
    assdraw
    PRIVATE
        _CRT_SECURE_NO_WARNINGS
        NOMINMAX
        UNICODE
        _UNICODE
)

target_link_libraries(
    assdraw
    PRIVATE
        assdraw_agg
        ${ASSDRAW_WX_LIBRARIES}
)

set_target_properties(
    assdraw
    PROPERTIES
        OUTPUT_NAME assdraw3
        VS_DEBUGGER_WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
)

install(TARGETS assdraw RUNTIME DESTINATION .)

set(ASSDRAW_LANGUAGES tr fr de es zh_TW vi)
set(ASSDRAW_MO_FILES)
foreach(language IN LISTS ASSDRAW_LANGUAGES)
    set(mo "${CMAKE_CURRENT_BINARY_DIR}/locale/${language}/LC_MESSAGES/assdraw.mo")
    configure_file("${CMAKE_CURRENT_SOURCE_DIR}/vendor/wxlocale/${language}/LC_MESSAGES/wxstd.mo"
        "${CMAKE_CURRENT_BINARY_DIR}/locale/${language}/LC_MESSAGES/wxstd.mo" COPYONLY)
    add_custom_command(OUTPUT "${mo}"
        COMMAND ${Python3_EXECUTABLE} "${CMAKE_CURRENT_SOURCE_DIR}/tools/translations.py"
            --compile-one "${CMAKE_CURRENT_SOURCE_DIR}/po/${language}.po" "${mo}"
        DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/po/${language}.po"
            "${CMAKE_CURRENT_SOURCE_DIR}/tools/translations.py"
        VERBATIM)
    list(APPEND ASSDRAW_MO_FILES "${mo}")
endforeach()
add_custom_target(assdraw-translations ALL DEPENDS ${ASSDRAW_MO_FILES})
install(DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/locale" DESTINATION .)
install(FILES docs/ASSDraw3.chm LICENSE NOTICE.md README.md README.tr.md DESTINATION .)
install(FILES vendor/wxlocale/LICENSE.wxWidgets.txt vendor/wxlocale/COPYING.LIB.txt DESTINATION .)
install(FILES vendor/fontawesome/LICENSE.txt DESTINATION licenses RENAME FontAwesome.txt)
install(FILES vendor/fontawesome/README.md DESTINATION licenses RENAME FontAwesome-attribution.md)
install(DIRECTORY docs/help DESTINATION .)

if(MSVC)
    set(CMAKE_INSTALL_SYSTEM_RUNTIME_LIBS_SKIP TRUE)
    include(InstallRequiredSystemLibraries)
    install(PROGRAMS ${CMAKE_INSTALL_SYSTEM_RUNTIME_LIBS} DESTINATION .)
endif()

if(BUILD_TESTING)
    add_executable(assdraw-gui-tests tests/canvas_tests.cpp ${ASSDRAW_SOURCES} ${ASSDRAW_RESOURCES})
    target_include_directories(assdraw-gui-tests PRIVATE src src/wxAGG src/xpm)
    target_compile_definitions(assdraw-gui-tests PRIVATE ASSDRAW_NO_APP_MAIN NOMINMAX UNICODE _UNICODE
        ASSDRAW_SOURCE_DIR="${CMAKE_CURRENT_SOURCE_DIR}")
    target_link_libraries(assdraw-gui-tests PRIVATE assdraw_agg ${ASSDRAW_WX_LIBRARIES})
    if(MSVC)
        target_compile_options(assdraw-gui-tests PRIVATE /UNDEBUG)
    else()
        target_compile_options(assdraw-gui-tests PRIVATE -UNDEBUG)
    endif()
    if(UNIX AND NOT APPLE)
        find_program(XVFB_RUN xvfb-run)
        if(XVFB_RUN)
            add_test(NAME assdraw-gui-tests COMMAND ${XVFB_RUN} -a $<TARGET_FILE:assdraw-gui-tests>)
        endif()
    else()
        add_test(NAME assdraw-gui-tests COMMAND assdraw-gui-tests)
    endif()
    if(TEST assdraw-gui-tests)
        set_tests_properties(assdraw-gui-tests PROPERTIES TIMEOUT 90)
    endif()
endif()

foreach(app_target assdraw assdraw-gui-tests)
    if(TARGET ${app_target})
        add_dependencies(${app_target} assdraw-icons assdraw-bitmaps)
        target_include_directories(${app_target} PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/generated")
        if(UNIX AND NOT APPLE)
            target_link_libraries(${app_target} PRIVATE PkgConfig::ASSDRAW_GTK)
        endif()
        if(WIN32)
            target_link_libraries(${app_target} PRIVATE shell32)
        endif()
        add_dependencies(${app_target} assdraw-translations)
        if(MSVC)
            target_compile_options(${app_target} PRIVATE /utf-8)
            add_custom_command(TARGET ${app_target} POST_BUILD
                COMMAND ${CMAKE_COMMAND} -E copy_if_different
                    ${CMAKE_INSTALL_SYSTEM_RUNTIME_LIBS} "$<TARGET_FILE_DIR:${app_target}>"
                COMMAND_EXPAND_LISTS VERBATIM)
        endif()
        foreach(language IN LISTS ASSDRAW_LANGUAGES)
            add_custom_command(TARGET ${app_target} POST_BUILD
                COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:${app_target}>/locale/${language}/LC_MESSAGES"
                COMMAND ${CMAKE_COMMAND} -E copy_if_different
                    "${CMAKE_CURRENT_BINARY_DIR}/locale/${language}/LC_MESSAGES/assdraw.mo"
                    "$<TARGET_FILE_DIR:${app_target}>/locale/${language}/LC_MESSAGES/assdraw.mo"
                COMMAND ${CMAKE_COMMAND} -E copy_if_different
                    "${CMAKE_CURRENT_BINARY_DIR}/locale/${language}/LC_MESSAGES/wxstd.mo"
                    "$<TARGET_FILE_DIR:${app_target}>/locale/${language}/LC_MESSAGES/wxstd.mo"
                VERBATIM)
        endforeach()
        add_custom_command(TARGET ${app_target} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different "${CMAKE_CURRENT_SOURCE_DIR}/docs/ASSDraw3.chm" "$<TARGET_FILE_DIR:${app_target}>/ASSDraw3.chm"
            COMMAND ${CMAKE_COMMAND} -E copy_directory "${CMAKE_CURRENT_SOURCE_DIR}/docs/help" "$<TARGET_FILE_DIR:${app_target}>/help"
            VERBATIM)
    endif()
endforeach()
