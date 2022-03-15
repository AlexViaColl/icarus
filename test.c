#include <X11/Xlib.h>
#include <vulkan/vulkan.h>
#include <vulkan/vulkan_xlib.h>

#include <stdio.h>
#include <stdlib.h>

int main(void) {
    VkInstance instance = 0;
    VkApplicationInfo application_info = {
        .sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
        //.pApplicationName = "Test",
        //.pEngineName = "No Engine"
    };
    const char* layer_names[] = {"VK_LAYER_KHRONOS_validation"};
    const char* extension_names[] = {
        VK_KHR_SURFACE_EXTENSION_NAME,
        VK_KHR_XLIB_SURFACE_EXTENSION_NAME,
    };
    VkInstanceCreateInfo instance_create_info = {
        .sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        .pApplicationInfo = &application_info,
        .enabledLayerCount = 1,
        .ppEnabledLayerNames = layer_names,
        .enabledExtensionCount = 2,
        .ppEnabledExtensionNames = extension_names,
    };
    if (vkCreateInstance(
                &instance_create_info,
                0,
                &instance
                ) != VK_SUCCESS) {
        fprintf(stderr, "Failed to create instance\n");
        return 1;
    }

    XInitThreads();
    Display *display = XOpenDisplay(0);
    if (!display) {
        fprintf(stderr, "Cannot open display\n");
        return 1;
    }

    int screen = XDefaultScreen(display);
    Window root = XRootWindow(display, screen);
    Window window = XCreateSimpleWindow(display, root, 0, 0, 800, 600, 1, 0, 0);

    VkSurfaceKHR surface = 0;
    VkXlibSurfaceCreateInfoKHR surface_create_info = {
        .sType = VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
        .dpy = display,
        .window = window,
    };
    if(vkCreateXlibSurfaceKHR(
                instance,
                &surface_create_info,
                0,
                &surface
                ) != VK_SUCCESS) {
        fprintf(stderr, "Failed to create Xlib surface\n");
        return 1;
    }

    int device_count = 0;
    vkEnumeratePhysicalDevices(instance, &device_count, 0); 
    VkPhysicalDevice *physical_devices = (VkPhysicalDevice*)malloc(device_count * sizeof(VkPhysicalDevice));
    vkEnumeratePhysicalDevices(instance, &device_count, physical_devices);

    VkBool32 present_support = 0;
    for (int i = 0; i < device_count; i++) {
        vkGetPhysicalDeviceSurfaceSupportKHR(physical_devices[i], 0, surface, &present_support);
    }

    return 0;
}
