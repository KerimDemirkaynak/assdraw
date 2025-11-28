project('ASSDraw3', 'cpp',
  version : '3.0.0',
  default_options : ['warning_level=1', 'cpp_std=c++14'])

# -------------------------------------------------------------
# 1. BAĞIMLILIKLAR (wxWidgets)
# -------------------------------------------------------------

# Windows'ta vcpkg kullandıgımız icin 'cmake' yöntemini zorluyoruz.
# Linux'ta ise standart 'config-tool' (wx-config) kullanilabilir.
if host_machine.system() == 'windows'
    wx_dep = dependency('wxwidgets', 
        modules : ['core', 'base', 'net', 'propgrid'], 
        method : 'cmake', 
        required : true)
else
    wx_dep = dependency('wxwidgets', 
        modules : ['core', 'base', 'net', 'propgrid'], 
        required : true)
endif

# -------------------------------------------------------------
# 2. AGG KÜTÜPHANESİ (Manuel Kaynak Koddan)
# -------------------------------------------------------------
# vcpkg'de agg olmadigi icin, build.yml ile indirdigimiz 
# yerel 'agg-2.5' klasorunu kullaniyoruz.

agg_inc = include_directories('agg-2.5/include')

# Derlenmesi gereken AGG dosyalari
agg_src = files(
  'agg-2.5/src/agg_arc.cpp',
  'agg-2.5/src/agg_arrowhead.cpp',
  'agg-2.5/src/agg_bezier_arc.cpp',
  'agg-2.5/src/agg_bspline.cpp',
  'agg-2.5/src/agg_curves.cpp',
  'agg-2.5/src/agg_vcgen_contour.cpp',
  'agg-2.5/src/agg_vcgen_dash.cpp',
  'agg-2.5/src/agg_vcgen_markers_term.cpp',
  'agg-2.5/src/agg_vcgen_smooth_poly1.cpp',
  'agg-2.5/src/agg_vcgen_stroke.cpp',
  'agg-2.5/src/agg_gsv_text.cpp',
  'agg-2.5/src/agg_image_filters.cpp',
  'agg-2.5/src/agg_line_aa_basics.cpp',
  'agg-2.5/src/agg_line_profile_aa.cpp',
  'agg-2.5/src/agg_rounded_rect.cpp',
  'agg-2.5/src/agg_sqrt_tables.cpp',
  'agg-2.5/src/agg_trans_affine.cpp',
  'agg-2.5/src/agg_trans_warp_magnifier.cpp',
  'agg-2.5/src/agg_vpgen_clip_polygon.cpp',
  'agg-2.5/src/agg_vpgen_clip_polyline.cpp',
  'agg-2.5/src/agg_vpgen_segmentator.cpp'
)

# -------------------------------------------------------------
# 3. PROJE KAYNAK DOSYALARI
# -------------------------------------------------------------
sources = [
  'src/assdraw.cpp',
  'src/assdraw_settings.cpp',
  'src/agg_bcspline.cpp',
  'src/agg_vcgen_bcspline.cpp',
  'src/canvas.cpp',
  'src/canvas_mouse.cpp',
  'src/cmd.cpp',
  'src/dlgctrl.cpp',
  'src/engine.cpp',
  'src/library.cpp',
  'src/settings.cpp',
  'src/wxAGG/AGGWindow.cpp',
]

# Windows için ikon ve manifest dosyasi (.rc)
if host_machine.system() == 'windows'
  windows = import('windows')
  sources += windows.compile_resources('src/assdraw.rc')
endif

# -------------------------------------------------------------
# 4. ÇIKTI (Executable)
# -------------------------------------------------------------
executable('ASSDraw3',
  sources + agg_src,
  include_directories : [agg_inc],
  dependencies : [wx_dep],
  install : true,
  win_subsystem : 'windows') 
  # win_subsystem: konsol penceresi acilmasini engeller
