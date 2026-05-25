cmake_minimum_required(VERSION 3.10)
project(fastfetch_sys_vendor C CXX ASM)

add_subdirectory(vendor)

add_library(fastfetch-vendor)
target_link_libraries(fastfetch-vendor libfastfetch)
