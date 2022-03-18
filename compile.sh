#!/bin/sh

# compile our shaders
glslc shader.vert -o vert.spv
glslc shader.frag -o frag.spv

# compile stb_image
gcc -o stb_image.o -c stb_image.c
ar rcs libstb_image.a stb_image.o
