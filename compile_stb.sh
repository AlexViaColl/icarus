#!/bin/bash

set -e

echo "Compiling stb_image..."
gcc -o stb_image.o -c stb_image.c
ar rcs libstb_image.a stb_image.o
