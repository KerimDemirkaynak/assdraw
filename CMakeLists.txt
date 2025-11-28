cmake_minimum_required(VERSION 3.10)
project(ASSDraw3)

# C++ Standartlarını belirle (Modern C++ için)
set(CMAKE_CXX_STANDARD 14)

# vcpkg'den gelen kütüphaneleri bul
find_package(wxWidgets REQUIRED COMPONENTS core base net propgrid)
find_package(unofficial-agg CONFIG REQUIRED)

# Kaynak kodlarını otomatik bul (src klasörü altındakiler)
file(GLOB_RECURSE SOURCES 
    "src/*.cpp" 
    "src/*.h" 
    "src/*.hpp"
)

# Windows Kaynak Dosyası (İkonlar vb. için)
if(WIN32)
    list(APPEND SOURCES "src/assdraw.rc")
endif()

# Uygulamayı tanımla (Windows uygulaması olduğu için WIN32 ekliyoruz)
add_executable(ASSDraw3 WIN32 ${SOURCES})

# Include (Başlık) yollarını ekle
target_include_directories(ASSDraw3 PRIVATE src)

# Kütüphaneleri projeye bağla
target_link_libraries(ASSDraw3 PRIVATE 
    wx::core wx::base wx::net wx::propgrid 
    unofficial-agg::agg
)
