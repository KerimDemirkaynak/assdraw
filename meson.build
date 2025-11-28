project('ASSDraw3', 'cpp',
  version : '3.0.0',
  default_options : ['warning_level=1', 'cpp_std=c++14'])

# Bağımlılıklar (Windows ve Linux farkları gözetilerek)
wx_dep = dependency('wxwidgets', modules : ['core', 'base', 'net', 'propgrid'], required : true)

# AGG için özel kontrol (Meson'un wrapdb'sinde olmayabilir, sistemde arar)
# Eğer Linux ise pkg-config, Windows ise manuel include/lib yolu gerekebilir.
# Basitlik için burada header-only varsayımı veya sistem kütüphanesi varsayımı yapıyoruz.
# Windows'ta vcpkg ile gelen AGG'yi bulmak için cpp.find_library kullanabiliriz.
cpp = meson.get_compiler('cpp')
agg_dep = dependency('libagg', required : false)
if not agg_dep.found()
  # Windows veya manuel kurulum fallback
  agg_dep = cpp.find_library('agg', required : true)
endif

# Kaynak Dosyalar
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
  # src klasöründeki diğer .cpp dosyalarını buraya ekle...
]

# Windows Kaynak Dosyası (.rc)
if host_machine.system() == 'windows'
  windows = import('windows')
  sources += windows.compile_resources('src/assdraw.rc')
endif

# Uygulamayı Oluştur
executable('ASSDraw3',
  sources,
  dependencies : [wx_dep, agg_dep],
  install : true)
