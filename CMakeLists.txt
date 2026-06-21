cmake_minimum_required(VERSION 3.25)

project(assdraw VERSION 3.0.0 LANGUAGES CXX RC)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(MSVC)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>DLL")
endif()

set(ASSDRAW_AGG_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/vendor/agg/agg-src")
if(NOT EXISTS "${ASSDRAW_AGG_ROOT}/include/agg_basics.h")
    message(
        FATAL_ERROR
        "AGG submodule was not found. Run `git submodule update --init vendor/agg` first."
    )
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

set(wxWidgets_USE_STATIC OFF)
set(ASSDRAW_WX_COMPONENTS base core adv aui html propgrid)
find_package(wxWidgets REQUIRED COMPONENTS ${ASSDRAW_WX_COMPONENTS})
include(${wxWidgets_USE_FILE})
set(ASSDRAW_WX_LIBRARIES ${wxWidgets_LIBRARIES})
find_package(WebP CONFIG REQUIRED)

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
        WebP::webp
        WebP::webpdecoder
        WebP::webpdemux
)

set_target_properties(
    assdraw
    PROPERTIES
        OUTPUT_NAME assdraw3
        VS_DEBUGGER_WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
)

install(TARGETS assdraw RUNTIME DESTINATION .)

# --- i18n (gettext) CMake Entegrasyonu ---
find_program(MSGFMT_EXECUTABLE msgfmt)
set(ASSDRAW_LINGUAS tr;en)
if(MSGFMT_EXECUTABLE)
    set(MO_FILES "")
    foreach(LANG ${ASSDRAW_LINGUAS})
        set(PO_FILE "${CMAKE_SOURCE_DIR}/po/${LANG}.po")
        set(MO_DIR "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/locale/${LANG}/LC_MESSAGES")
        set(MO_FILE "${MO_DIR}/assdraw.mo")
        add_custom_command(
            OUTPUT ${MO_FILE}
            COMMAND ${CMAKE_COMMAND} -E make_directory ${MO_DIR}
            COMMAND ${MSGFMT_EXECUTABLE} -o ${MO_FILE} ${PO_FILE}
            DEPENDS ${PO_FILE}
        )
        list(APPEND MO_FILES ${MO_FILE})
    endforeach()
    add_custom_target(translations ALL DEPENDS ${MO_FILES})
else()
    message(WARNING "msgfmt bulunamadı, .po dosyaları derlenmeyecek.")
endif()
