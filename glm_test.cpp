#define GLM_FORCE_RADIANS
#define GLM_FORCE_DEPTH_ZERO_TO_ONE
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>

#include <stdio.h>

void dump_mat4(const char *name, glm::mat4 m) {
    printf("%s: {\n", name);
    printf("    %f %f %f %f\n", m[0][0], m[0][1], m[0][2], m[0][3]);
    printf("    %f %f %f %f\n", m[1][0], m[1][1], m[1][2], m[1][3]);
    printf("    %f %f %f %f\n", m[2][0], m[2][1], m[2][2], m[2][3]);
    printf("    %f %f %f %f\n", m[3][0], m[3][1], m[3][2], m[3][3]);
    printf("}\n");
}

int main() {
    //auto model = glm::rotate(glm::mat4(1.0f), time * glm::radians(90.0f), glm::vec3(0.0f, 0.0f, 1.0f));
    auto view = glm::lookAt(glm::vec3(2.0f, 2.0f, 2.0f), glm::vec3(0.0f, 0.0f, 0.0f), glm::vec3(0.0f, 0.0f, 1.0f));
    auto proj = glm::perspective(glm::radians(45.0f),  1200.0f / 675.0f, 0.1f, 10.0f);
    proj[1][1] *= -1;

    dump_mat4("View", view);
    dump_mat4("Proj", proj);
}
