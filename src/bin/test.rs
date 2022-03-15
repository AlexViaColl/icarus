use icarus::*;

use core::ffi::c_void;
use std::ffi::CStr;
use std::fs;
use std::process;
use std::ptr;

macro_rules! check(
    ($expression:expr) => {
        assert_eq!($expression, VK_SUCCESS);
    }
);

fn main() {
    unsafe {
        let mut instance = ptr::null_mut();
        check!(vkCreateInstance(
            &VkInstanceCreateInfo {
                sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                pApplicationInfo: &VkApplicationInfo {
                    sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
                    pNext: ptr::null(),
                    pApplicationName: b"Hello Triangle\0".as_ptr() as *const i8,
                    applicationVersion: 0,
                    pEngineName: b"No Engine\0".as_ptr() as *const i8,
                    engineVersion: 0,
                    apiVersion: 0,
                },
                enabledLayerCount: 1,
                ppEnabledLayerNames: [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8].as_ptr(),
                enabledExtensionCount: 3,
                ppEnabledExtensionNames: [
                    VK_KHR_SURFACE_EXTENSION_NAME,
                    VK_KHR_XLIB_SURFACE_EXTENSION_NAME,
                    VK_EXT_DEBUG_UTILS_EXTENSION_NAME,
                ]
                .as_ptr(),
            },
            ptr::null(),
            &mut instance,
        ));

        XInitThreads();
        let display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            eprintln!("Cannot open display");
            process::exit(1);
        }

        let screen = XDefaultScreen(display);
        let root = XRootWindow(display, screen);
        let window = XCreateSimpleWindow(display, root, 0, 0, 800, 600, 1, 0, 0);

        assert_ne!(XStoreName(display, window, b"Icarus\0".as_ptr() as *const i8), 0);
        assert_ne!(XSelectInput(display, window, KeyPressMask | ExposureMask | StructureNotifyMask), 0);
        assert_ne!(XMapWindow(display, window), 0);

        let mut surface = ptr::null_mut();
        check!(vkCreateXlibSurfaceKHR(
            instance,
            &VkXlibSurfaceCreateInfoKHR {
                sType: VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
                pNext: ptr::null(),
                flags: 0,
                dpy: display,
                window,
            },
            ptr::null(),
            &mut surface,
        ));

        let mut device_count = 0;
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut()));
        println!("Devices ({}):", device_count);
        assert_ne!(device_count, 0);
        let mut physical_devices = vec![ptr::null_mut(); device_count as usize];
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr()));

        let mut present_support = 0;
        vkGetPhysicalDeviceSurfaceSupportKHR(physical_devices[0], 0, surface, &mut present_support);
        vkGetPhysicalDeviceSurfaceSupportKHR(physical_devices[1], 0, surface, &mut present_support);
    }
}
