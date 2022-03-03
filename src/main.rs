#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
use icarus::*;

use core::ffi::c_void;
use std::ffi::CStr;
use std::process;
use std::ptr;

const BG_COLOR: u64 = 0x00000000;

extern "C" fn error_handler(_display: *mut Display, _event: *mut XErrorEvent) -> i32 {
    println!("An error ocurred!");
    0
}
extern "C" fn error_io_handler(_display: *mut Display) -> i32 {
    panic!("A fatal I/O error ocurred!");
}

fn main() {
    unsafe {
        let mut extension_count = 0;
        vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, ptr::null_mut());

        let mut extensions = vec![VkExtensionProperties::default(); extension_count as usize];
        vkEnumerateInstanceExtensionProperties(
            ptr::null(),
            &mut extension_count,
            extensions.as_mut_ptr(),
        );
        println!("Extensions ({}):", extension_count);
        for extension in &extensions {
            println!("{:?}", CStr::from_ptr(extension.extensionName.as_ptr()));
        }

        let mut layer_count = 0;
        vkEnumerateInstanceLayerProperties(&mut layer_count, ptr::null_mut());
        let mut layers = vec![VkLayerProperties::default(); layer_count as usize];
        vkEnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr());
        println!("\nLayers ({}):", layer_count);
        for layer in &layers {
            println!(
                "{:?}: {:?}",
                CStr::from_ptr(layer.layerName.as_ptr()),
                CStr::from_ptr(layer.layerName.as_ptr()),
            );
        }

        let mut instance = ptr::null_mut();
        let result = vkCreateInstance(
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
                ppEnabledLayerNames: [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8]
                    .as_ptr(),
                enabledExtensionCount: 3,
                ppEnabledExtensionNames: [
                    b"VK_KHR_surface\0".as_ptr() as *const i8,
                    b"VK_KHR_xlib_surface\0".as_ptr() as *const i8,
                    b"VK_EXT_debug_utils\0".as_ptr() as *const i8, // Actually defined as VK_EXT_DEBUG_UTILS_EXTENSION_NAME
                ]
                .as_ptr(),
            },
            ptr::null(),
            &mut instance,
        );
        println!("vkCreateInstance result: {:?}", result);

        let mut debug_messenger = ptr::null_mut();
        let func = vkGetInstanceProcAddr(
            instance,
            b"vkCreateDebugUtilsMessengerEXT\0".as_ptr() as *const i8,
        );
        let func = std::mem::transmute::<_, PFN_vkCreateDebugUtilsMessengerEXT>(func);
        println!("vkCreateDebugUtilsMessengerEXT: {:?}", func);
        let result = func(
            instance,
            &VkDebugUtilsMessengerCreateInfoEXT {
                sType: VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                pNext: ptr::null(),
                flags: 0,
                messageSeverity: 0, //VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                //                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                //                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT,
                messageType: 0, // VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                //                    | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                //                    | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT,
                pfnUserCallback: debug_callback,
                pUserData: ptr::null_mut(),
            },
            ptr::null(),
            &mut debug_messenger,
        );
        println!(
            "result: {:?}, debug_messenger: {:?}",
            result, debug_messenger
        );

        vkDestroyInstance(instance, ptr::null());
        process::exit(1);

        let display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            eprintln!("Cannot open display");
            process::exit(1);
        }

        let _orig_err_handler = XSetErrorHandler(error_handler);
        let _orig_err_io_handler = XSetIOErrorHandler(error_io_handler);

        let screen = XDefaultScreen(display);
        let root = XRootWindow(display, screen);
        let window = XCreateSimpleWindow(display, root, 0, 0, 100, 100, 1, 0, BG_COLOR);

        assert_ne!(
            XStoreName(display, window, b"Icarus\0".as_ptr() as *const i8),
            0
        );
        assert_ne!(
            XSelectInput(display, window, KeyPressMask | ExposureMask),
            0
        );
        assert_ne!(XMapWindow(display, window), 0);

        let mut running = true;
        while running {
            while XPending(display) > 0 {
                let mut event = XEvent { pad: [0; 24] };
                XNextEvent(display, &mut event);
                match event.ttype {
                    KeyPress => {
                        let keysym = XLookupKeysym(&mut event.xkey, 0);
                        let event = event.xkey;
                        println!("KeySym: {} / KeyCode: {}", keysym, event.keycode);
                        match event.keycode {
                            9 => running = false,
                            n => println!("Keycode: {}", n),
                        }
                    }
                    Expose => {
                        // let gc = XDefaultGC(display, screen);
                        // XFillRectangle(display, window, gc, 20, 20, 10, 10);
                    }
                    _ => {}
                }
            }
        }

        XCloseDisplay(display);
    };
}

extern "C" fn debug_callback(
    message_severity: VkDebugUtilsMessageSeverityFlagsEXT,
    message_type: VkDebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const VkDebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut c_void,
) -> VkBool32 {
    println!("Inside debug_callback");
    0
}
