#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
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

macro_rules! check(
    ($expression:expr) => {
        assert_eq!($expression, VK_SUCCESS);
    }
);

fn cstr_to_string(ptr: *const i8) -> String {
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    unsafe {
        // Vulkan initialization
        let mut extension_count = 0;
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, ptr::null_mut()));

        let mut extensions = vec![VkExtensionProperties::default(); extension_count as usize];
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, extensions.as_mut_ptr(),));
        println!("Extensions ({}):", extension_count);
        for extension in &extensions {
            println!("{}", cstr_to_string(extension.extensionName.as_ptr()));
        }
        println!();

        let mut layer_count = 0;
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, ptr::null_mut()));
        let mut layers = vec![VkLayerProperties::default(); layer_count as usize];
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr()));
        println!("Layers ({}):", layer_count);
        for layer in &layers {
            println!("{}: {}", cstr_to_string(layer.layerName.as_ptr()), cstr_to_string(layer.description.as_ptr()));
        }
        println!();

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

        let mut debug_messenger = ptr::null_mut();
        let vkCreateDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkCreateDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, b"vkCreateDebugUtilsMessengerEXT\0".as_ptr() as *const i8),
        );
        check!(vkCreateDebugUtilsMessengerEXT(
            instance,
            &VkDebugUtilsMessengerCreateInfoEXT {
                sType: VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                pNext: ptr::null(),
                flags: 0,
                messageSeverity: 0 //| VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                    //| VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT,
                messageType: VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT,
                pfnUserCallback: debug_callback,
                pUserData: ptr::null_mut(),
            },
            ptr::null(),
            &mut debug_messenger,
        ));

        // pick physical device
        let mut device_count = 0;
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut()));
        println!("Devices ({}):", device_count);
        assert_ne!(device_count, 0);
        let mut physical_devices = vec![ptr::null_mut(); device_count as usize];
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr()));
        for device in &physical_devices {
            let mut properties = VkPhysicalDeviceProperties::default();
            vkGetPhysicalDeviceProperties(*device, &mut properties);
            println!("{}", cstr_to_string(properties.deviceName.as_ptr()));
            // println!("{:#?}", properties);

            let mut features = VkPhysicalDeviceFeatures::default();
            vkGetPhysicalDeviceFeatures(*device, &mut features);
            // println!("{:#?}", features);

            let mut queue_family_count = 0;
            vkGetPhysicalDeviceQueueFamilyProperties(*device, &mut queue_family_count, ptr::null_mut());
            let mut queue_families = vec![VkQueueFamilyProperties::default(); queue_family_count as usize];
            vkGetPhysicalDeviceQueueFamilyProperties(*device, &mut queue_family_count, queue_families.as_mut_ptr());
            // println!("{:#?}", queue_families);
            for (_index, queue_family) in queue_families.iter().enumerate() {
                if (*queue_family).queueFlags & VK_QUEUE_GRAPHICS_BIT != 0 {
                    // println!("Found a queue {} with VK_QUEUE_GRAPHICS_BIT", _index);
                }
            }
        }
        // TODO: Score physical devices and pick the "best" one.
        // TODO: Prefer dedicated gpu over integrated.
        // TODO: The chosen gpu should have at least one queue family supporting graphics.
        let physical_device = physical_devices[0]; // Pick the first physical device for now.

        let graphics_family_index = 0; // TODO: Actually grab this

        let mut device = ptr::null_mut();
        check!(vkCreateDevice(
            physical_device,
            &VkDeviceCreateInfo {
                sType: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                queueCreateInfoCount: 1,
                pQueueCreateInfos: [VkDeviceQueueCreateInfo {
                    sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    queueFamilyIndex: graphics_family_index,
                    queueCount: 1,
                    pQueuePriorities: [1.0].as_ptr(),
                }]
                .as_ptr(),
                enabledLayerCount: 0,
                ppEnabledLayerNames: ptr::null(),
                enabledExtensionCount: 0,
                ppEnabledExtensionNames: ptr::null(),
                pEnabledFeatures: &VkPhysicalDeviceFeatures::default(),
            },
            ptr::null(),
            &mut device,
        ));

        let mut graphics_queue = ptr::null_mut();
        vkGetDeviceQueue(device, graphics_family_index, 0, &mut graphics_queue);
        println!("Queue: {:?}", graphics_queue);

        // Cleanup
        vkDestroyDevice(device, ptr::null());

        // destroy debug_messenger
        let vkDestroyDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, b"vkDestroyDebugUtilsMessengerEXT\0".as_ptr() as *const i8),
        );
        vkDestroyDebugUtilsMessengerEXT(instance, debug_messenger, ptr::null());

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

        assert_ne!(XStoreName(display, window, b"Icarus\0".as_ptr() as *const i8), 0);
        assert_ne!(XSelectInput(display, window, KeyPressMask | ExposureMask), 0);
        assert_ne!(XMapWindow(display, window), 0);

        let mut running = true;
        while running {
            while XPending(display) > 0 {
                let mut event = XEvent {
                    pad: [0; 24],
                };
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
    _message_severity: VkDebugUtilsMessageSeverityFlagsEXT,
    _message_type: VkDebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const VkDebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> VkBool32 {
    unsafe {
        println!("{}", CStr::from_ptr((*p_callback_data).pMessage).to_string_lossy());
        0
    }
}
