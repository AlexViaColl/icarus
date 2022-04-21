#!/bin/bash

set -e

echo "Compiling Shaders..."
for FILE in $(find ./assets/shaders -name '*.vert' -o -name '*.frag'); do ./glslc $FILE -o $FILE.spv; done
