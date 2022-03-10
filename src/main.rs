#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
use icarus::*;

use core::ffi::c_void;
use std::ffi::CStr;
use std::fs;
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

        // create surface
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
        println!("Surface: {:?}", surface);

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
            for (index, queue_family) in queue_families.iter().enumerate() {
                if (*queue_family).queueFlags & VK_QUEUE_GRAPHICS_BIT != 0 {
                    // println!("Found a queue {} with VK_QUEUE_GRAPHICS_BIT", index);
                }
                let mut present_support = 0;
                vkGetPhysicalDeviceSurfaceSupportKHR(*device, index as u32, surface, &mut present_support);
            }
        }
        // TODO: Score physical devices and pick the "best" one.
        // TODO: Prefer dedicated gpu over integrated.
        // TODO: The chosen gpu should have at least one queue family supporting graphics.
        let physical_device = physical_devices[0]; // Pick the first physical device for now.

        let graphics_family_index = 0; // TODO: Actually grab this

        let mut present_support = 0;
        vkGetPhysicalDeviceSurfaceSupportKHR(physical_device, graphics_family_index, surface, &mut present_support);
        if present_support != 0 {
            println!("Queue supports presentation and graphics operations.");
        }

        // create logical device
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
                enabledExtensionCount: 1,
                // TODO: Check that extension is actually supported
                ppEnabledExtensionNames: [VK_KHR_SWAPCHAIN_EXTENSION_NAME].as_ptr(),
                pEnabledFeatures: &VkPhysicalDeviceFeatures::default(),
            },
            ptr::null(),
            &mut device,
        ));

        // We are assuming this queue supports presentation to the surface as well!
        let mut graphics_queue = ptr::null_mut();
        vkGetDeviceQueue(device, graphics_family_index, 0, &mut graphics_queue);

        let mut surface_caps = VkSurfaceCapabilitiesKHR::default();
        check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface, &mut surface_caps));
        println!("{:#?}", surface_caps);

        let mut surface_formats_count = 0;
        check!(vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface,
            &mut surface_formats_count,
            ptr::null_mut()
        ));
        let mut surface_formats = vec![VkSurfaceFormatKHR::default(); surface_formats_count as usize];
        check!(vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface,
            &mut surface_formats_count,
            surface_formats.as_mut_ptr()
        ));
        println!("{:#?}", surface_formats);

        let mut surface_present_modes_count = 0;
        check!(vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface,
            &mut surface_present_modes_count,
            ptr::null_mut()
        ));
        let mut surface_present_modes = vec![VkPresentModeKHR::default(); surface_present_modes_count as usize];
        check!(vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface,
            &mut surface_present_modes_count,
            surface_present_modes.as_mut_ptr()
        ));
        println!("{:#?}", surface_present_modes);

        assert!(surface_formats.contains(&VkSurfaceFormatKHR {
            format: VK_FORMAT_B8G8R8A8_SRGB,
            colorSpace: VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
        }));
        assert!(surface_present_modes.contains(&VK_PRESENT_MODE_MAILBOX_KHR));

        let mut swapchain = ptr::null_mut();
        check!(vkCreateSwapchainKHR(
            device,
            &VkSwapchainCreateInfoKHR {
                sType: VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
                pNext: ptr::null(),
                flags: 0,
                surface,
                minImageCount: surface_caps.minImageCount + 1,
                imageFormat: VK_FORMAT_B8G8R8A8_SRGB,
                imageColorSpace: VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                imageExtent: surface_caps.currentExtent,
                imageArrayLayers: 1,
                imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
                imageSharingMode: VK_SHARING_MODE_EXCLUSIVE,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
                preTransform: surface_caps.currentTransform,
                compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
                presentMode: VK_PRESENT_MODE_MAILBOX_KHR,
                clipped: VK_TRUE,
                oldSwapchain: ptr::null_mut(),
            },
            ptr::null(),
            &mut swapchain
        ));

        let mut swapchain_image_count = 0;
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, ptr::null_mut()));
        let mut swapchain_images = vec![ptr::null_mut(); swapchain_image_count as usize];
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, swapchain_images.as_mut_ptr()));
        println!("Swapchain created with {} images", swapchain_image_count);

        let mut swapchain_image_views = vec![ptr::null_mut(); swapchain_images.len()];
        for i in 0..swapchain_image_views.len() {
            vkCreateImageView(
                device,
                &VkImageViewCreateInfo {
                    sType: VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    image: swapchain_images[i],
                    viewType: VK_IMAGE_VIEW_TYPE_2D,
                    format: VK_FORMAT_B8G8R8A8_SRGB,
                    components: VkComponentMapping {
                        r: VK_COMPONENT_SWIZZLE_IDENTITY,
                        g: VK_COMPONENT_SWIZZLE_IDENTITY,
                        b: VK_COMPONENT_SWIZZLE_IDENTITY,
                        a: VK_COMPONENT_SWIZZLE_IDENTITY,
                    },
                    subresourceRange: VkImageSubresourceRange {
                        aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                        baseMipLevel: 0,
                        levelCount: 1,
                        baseArrayLayer: 0,
                        layerCount: 1,
                    },
                },
                ptr::null(),
                &mut swapchain_image_views[i],
            );
        }

        // Create graphics pipeline
        let vs_code = fs::read("vert.spv").expect("Failed to load vertex shader");
        let fs_code = fs::read("frag.spv").expect("Failed to load fragment shader");

        let mut vs_shader_module = ptr::null_mut();
        check!(vkCreateShaderModule(
            device,
            &VkShaderModuleCreateInfo {
                sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                codeSize: vs_code.len(),
                pCode: vs_code.as_ptr() as *const u32,
            },
            ptr::null(),
            &mut vs_shader_module
        ));
        let mut fs_shader_module = ptr::null_mut();
        check!(vkCreateShaderModule(
            device,
            &VkShaderModuleCreateInfo {
                sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                codeSize: fs_code.len(),
                pCode: fs_code.as_ptr() as *const u32,
            },
            ptr::null(),
            &mut fs_shader_module
        ));

        let _shader_stages = [
            VkPipelineShaderStageCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                stage: VK_SHADER_STAGE_VERTEX_BIT,
                module: vs_shader_module,
                pName: b"main\0".as_ptr() as *const i8,
                pSpecializationInfo: ptr::null(),
            },
            VkPipelineShaderStageCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                stage: VK_SHADER_STAGE_FRAGMENT_BIT,
                module: fs_shader_module,
                pName: b"main\0".as_ptr() as *const i8,
                pSpecializationInfo: ptr::null(),
            },
        ];

        let _vertex_input_info = VkPipelineVertexInputStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            vertexBindingDescriptionCount: 0,
            pVertexBindingDescriptions: ptr::null(),
            vertexAttributeDescriptionCount: 0,
            pVertexAttributeDescriptions: ptr::null(),
        };
        let _input_assembly = VkPipelineInputAssemblyStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            topology: VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            primitiveRestartEnable: VK_FALSE,
        };
        let viewport = VkViewport {
            x: 0.0,
            y: 0.0,
            width: surface_caps.currentExtent.width as f32,
            height: surface_caps.currentExtent.height as f32,
            minDepth: 0.0,
            maxDepth: 1.0,
        };
        let scissor = VkRect2D {
            offset: VkOffset2D {
                x: 0,
                y: 0,
            },
            extent: surface_caps.currentExtent,
        };
        let _viewport_state = VkPipelineViewportStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            viewportCount: 1,
            pViewports: &viewport,
            scissorCount: 1,
            pScissors: &scissor,
        };
        let _rasterizer = VkPipelineRasterizationStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            depthClampEnable: VK_FALSE,
            rasterizerDiscardEnable: VK_FALSE,
            polygonMode: VK_POLYGON_MODE_FILL,
            cullMode: VK_CULL_MODE_BACK_BIT,
            frontFace: VK_FRONT_FACE_CLOCKWISE,
            depthBiasEnable: VK_FALSE,
            depthBiasConstantFactor: 0.0,
            depthBiasClamp: 0.0,
            depthBiasSlopeFactor: 0.0,
            lineWidth: 1.0,
        };
        let _multisampling = VkPipelineMultisampleStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            rasterizationSamples: VK_SAMPLE_COUNT_1_BIT,
            sampleShadingEnable: VK_FALSE,
            minSampleShading: 1.0,
            pSampleMask: ptr::null(),
            alphaToCoverageEnable: VK_FALSE,
            alphaToOneEnable: VK_FALSE,
        };
        let _color_blending = VkPipelineColorBlendStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            logicOpEnable: VK_FALSE,
            logicOp: VK_LOGIC_OP_COPY,
            attachmentCount: 1,
            pAttachments: &VkPipelineColorBlendAttachmentState {
                blendEnable: VK_TRUE,
                srcColorBlendFactor: VK_BLEND_FACTOR_SRC_ALPHA,
                dstColorBlendFactor: VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
                colorBlendOp: VK_BLEND_OP_ADD,
                srcAlphaBlendFactor: VK_BLEND_FACTOR_ONE,
                dstAlphaBlendFactor: VK_BLEND_FACTOR_ZERO,
                alphaBlendOp: VK_BLEND_OP_ADD,
                colorWriteMask: VK_COLOR_COMPONENT_R_BIT
                    | VK_COLOR_COMPONENT_G_BIT
                    | VK_COLOR_COMPONENT_B_BIT
                    | VK_COLOR_COMPONENT_A_BIT,
            },
            blendConstants: [0.0; 4],
        };
        // We can specify a few properties dynamically without having to recreate the pipeline.
        let _dynamic_state = VkPipelineDynamicStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            dynamicStateCount: 2,
            pDynamicStates: [VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_LINE_WIDTH].as_ptr(),
        };

        let mut pipeline_layout = ptr::null_mut();
        check!(vkCreatePipelineLayout(
            device,
            &VkPipelineLayoutCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                setLayoutCount: 0,
                pSetLayouts: ptr::null(),
                pushConstantRangeCount: 0,
                pPushConstantRanges: ptr::null(),
            },
            ptr::null(),
            &mut pipeline_layout
        ));

        // Cleanup
        vkDestroyPipelineLayout(device, pipeline_layout, ptr::null());

        vkDestroyShaderModule(device, fs_shader_module, ptr::null());
        vkDestroyShaderModule(device, vs_shader_module, ptr::null());

        for image_view in swapchain_image_views {
            vkDestroyImageView(device, image_view, ptr::null());
        }

        vkDestroySwapchainKHR(device, swapchain, ptr::null());

        vkDestroyDevice(device, ptr::null());

        // destroy debug_messenger
        let vkDestroyDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, b"vkDestroyDebugUtilsMessengerEXT\0".as_ptr() as *const i8),
        );
        vkDestroyDebugUtilsMessengerEXT(instance, debug_messenger, ptr::null());

        vkDestroySurfaceKHR(instance, surface, ptr::null());

        vkDestroyInstance(instance, ptr::null());
        process::exit(1);

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
