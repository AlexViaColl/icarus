#!/bin/bash

set -xe

# compile shaders
echo "Compiling Shaders..."
for FILE in $(find ./assets/shaders -name '*.vert' -o -name '*.frag'); do ./glslc $FILE -o $FILE.spv; done

# compile stb_image
echo "Compiling stb_image..."
gcc -o stb_image.o -c stb_image.c
ar rcs libstb_image.a stb_image.o
