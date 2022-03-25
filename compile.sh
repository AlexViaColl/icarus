#!/bin/sh

# compile our shaders
glslc assets/shaders/shader.vert -o assets/shaders/shader.vert.spv
glslc assets/shaders/shader.frag -o assets/shaders/shader.frag.spv

# compile stb_image
gcc -o stb_image.o -c stb_image.c
ar rcs libstb_image.a stb_image.o
