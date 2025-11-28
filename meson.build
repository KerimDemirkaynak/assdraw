project('ASSDraw3', 'cpp',
  version : '3.0.0',
  default_options : ['warning_level=1', 'cpp_std=c++14'])

# wxWidgets bağımlılığı
wx_dep = dependency('wxwidgets', modules : ['core', 'base', 'net', 'propgrid'], required : true)

# AGG bağımlılığı (agg veya unofficial-agg olarak arar)
cpp = meson.get_compiler('cpp')
agg_dep = dependency('libagg', required : false)

if not agg_dep.found()
    # Önce 'agg' ismini dene
    agg_dep = cpp.find_library('agg', required : false)
endif

if not agg_dep.found()
    # Bulamazsa 'unofficial-agg' ismini dene (vcpkg için)
    agg_dep = cpp.find_library('unofficial-agg', required : true)
endif

# Kaynak dosyalar
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
  # Diğer kaynak dosyaları buraya eklenebilir...
]

# Windows Kaynak Dosyası (.rc)
if host_machine.system() == 'windows'
  windows = import('windows')
  sources += windows.compile_resources('src/assdraw.rc')
endif

executable('ASSDraw3',
  sources,
  dependencies : [wx_dep, agg_dep],
  install : true)
