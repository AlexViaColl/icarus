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
const MAX_FRAMES_IN_FLIGHT: usize = 2;

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
        XInitThreads();
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

        let swapchain = create_swapchain(device, surface, surface_caps);
        let swapchain_image_views = create_image_views(device, swapchain);

        let render_pass = create_render_pass(device);
        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(device, render_pass, surface_caps);
        let framebuffers = create_framebuffers(device, render_pass, &swapchain_image_views, surface_caps);

        let mut swapchain_ctx = SwapchainContext {
            swapchain,
            image_views: swapchain_image_views,
            render_pass,
            pipeline_layout,
            graphics_pipeline,
            framebuffers,
        };

        // We can specify a few properties dynamically without having to recreate the pipeline.
        let _dynamic_state = VkPipelineDynamicStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            dynamicStateCount: 2,
            pDynamicStates: [VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_LINE_WIDTH].as_ptr(),
        };

        let mut command_pool = ptr::null_mut();
        check!(vkCreateCommandPool(
            device,
            &VkCommandPoolCreateInfo {
                sType: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
                pNext: ptr::null(),
                flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
                queueFamilyIndex: graphics_family_index,
            },
            ptr::null(),
            &mut command_pool
        ));

        let mut command_buffers = vec![ptr::null_mut(); MAX_FRAMES_IN_FLIGHT];
        check!(vkAllocateCommandBuffers(
            device,
            &VkCommandBufferAllocateInfo {
                sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
                pNext: ptr::null(),
                commandPool: command_pool,
                level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                commandBufferCount: command_buffers.len() as u32,
            },
            command_buffers.as_mut_ptr(),
        ));

        let mut image_available_semaphores = vec![ptr::null_mut(); MAX_FRAMES_IN_FLIGHT];
        let mut render_finished_semaphores = vec![ptr::null_mut(); MAX_FRAMES_IN_FLIGHT];
        let mut in_flight_fences = vec![ptr::null_mut(); MAX_FRAMES_IN_FLIGHT];
        let semaphore_create_info = VkSemaphoreCreateInfo {
            sType: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
        };
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            check!(vkCreateSemaphore(device, &semaphore_create_info, ptr::null(), &mut image_available_semaphores[i]));
            check!(vkCreateSemaphore(device, &semaphore_create_info, ptr::null(), &mut render_finished_semaphores[i]));
            check!(vkCreateFence(
                device,
                &VkFenceCreateInfo {
                    sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: VK_FENCE_CREATE_SIGNALED_BIT,
                },
                ptr::null(),
                &mut in_flight_fences[i],
            ));
        }

        // Main loop
        let mut running = true;
        let mut current_frame = 0;
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

            // draw
            check!(vkWaitForFences(device, 1, &in_flight_fences[current_frame], VK_TRUE, u64::MAX));

            let mut image_index = 0;
            match vkAcquireNextImageKHR(
                device,
                swapchain_ctx.swapchain,
                u64::MAX,
                image_available_semaphores[current_frame],
                ptr::null_mut(),
                &mut image_index,
            ) {
                VK_SUCCESS | VK_SUBOPTIMAL_KHR => {}
                VK_ERROR_OUT_OF_DATE_KHR => {
                    check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface, &mut surface_caps));
                    recreate_swapchain(&mut swapchain_ctx, device, surface, surface_caps);
                    continue;
                }
                res => panic!("{:?}", res),
            }

            check!(vkResetFences(device, 1, &in_flight_fences[current_frame]));

            vkResetCommandBuffer(command_buffers[current_frame], 0);

            // Record command buffer
            check!(vkBeginCommandBuffer(
                command_buffers[current_frame],
                &VkCommandBufferBeginInfo {
                    sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    pInheritanceInfo: ptr::null(),
                }
            ));

            vkCmdBeginRenderPass(
                command_buffers[current_frame],
                &VkRenderPassBeginInfo {
                    sType: VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
                    pNext: ptr::null(),
                    renderPass: swapchain_ctx.render_pass,
                    framebuffer: swapchain_ctx.framebuffers[image_index as usize],
                    renderArea: VkRect2D {
                        offset: VkOffset2D {
                            x: 0,
                            y: 0,
                        },
                        extent: surface_caps.currentExtent,
                    },
                    clearValueCount: 1,
                    pClearValues: &VkClearValue {
                        color: VkClearColorValue {
                            float32: [0.0, 0.0, 0.0, 1.0],
                        },
                    },
                },
                VK_SUBPASS_CONTENTS_INLINE,
            );

            vkCmdBindPipeline(
                command_buffers[current_frame],
                VK_PIPELINE_BIND_POINT_GRAPHICS,
                swapchain_ctx.graphics_pipeline,
            );

            vkCmdDraw(command_buffers[current_frame], 3, 1, 0, 0);

            vkCmdEndRenderPass(command_buffers[current_frame]);

            check!(vkEndCommandBuffer(command_buffers[current_frame]));

            // Submit command buffer
            check!(vkQueueSubmit(
                graphics_queue,
                1,
                &VkSubmitInfo {
                    sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
                    pNext: ptr::null(),
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &image_available_semaphores[current_frame],
                    pWaitDstStageMask: &VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                    commandBufferCount: 1,
                    pCommandBuffers: &command_buffers[current_frame],
                    signalSemaphoreCount: 1,
                    pSignalSemaphores: &render_finished_semaphores[current_frame],
                },
                in_flight_fences[current_frame],
            ));

            match vkQueuePresentKHR(
                graphics_queue,
                &VkPresentInfoKHR {
                    sType: VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
                    pNext: ptr::null(),
                    waitSemaphoreCount: 1,
                    pWaitSemaphores: &render_finished_semaphores[current_frame],
                    swapchainCount: 1,
                    pSwapchains: &swapchain_ctx.swapchain,
                    pImageIndices: &image_index,
                    pResults: ptr::null_mut(),
                },
            ) {
                VK_SUCCESS => {}
                VK_SUBOPTIMAL_KHR | VK_ERROR_OUT_OF_DATE_KHR => {
                    check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface, &mut surface_caps));
                    recreate_swapchain(&mut swapchain_ctx, device, surface, surface_caps);
                }
                res => panic!("{:?}", res),
            }

            current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }

        check!(vkDeviceWaitIdle(device));

        // Cleanup
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vkDestroyFence(device, in_flight_fences[i], ptr::null());
            vkDestroySemaphore(device, render_finished_semaphores[i], ptr::null());
            vkDestroySemaphore(device, image_available_semaphores[i], ptr::null());
        }
        vkDestroyCommandPool(device, command_pool, ptr::null());
        cleanup_swapchain(device, &mut swapchain_ctx);

        vkDestroyDevice(device, ptr::null());

        // destroy debug_messenger
        let vkDestroyDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, b"vkDestroyDebugUtilsMessengerEXT\0".as_ptr() as *const i8),
        );
        vkDestroyDebugUtilsMessengerEXT(instance, debug_messenger, ptr::null());

        vkDestroySurfaceKHR(instance, surface, ptr::null());

        // We need to close the display before destroying the vulkan instance to avoid segfaults!
        XCloseDisplay(display);
        vkDestroyInstance(instance, ptr::null());
    };
}

struct SwapchainContext {
    swapchain: VkSwapchainKHR,
    image_views: Vec<VkImageView>,
    render_pass: VkRenderPass,
    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,
    framebuffers: Vec<VkFramebuffer>,
}

fn recreate_swapchain(
    swapchain_ctx: &mut SwapchainContext,
    device: VkDevice,
    surface: VkSurfaceKHR,
    surface_caps: VkSurfaceCapabilitiesKHR,
) {
    unsafe {
        vkDeviceWaitIdle(device);

        cleanup_swapchain(device, swapchain_ctx);

        swapchain_ctx.swapchain = create_swapchain(device, surface, surface_caps);
        swapchain_ctx.image_views = create_image_views(device, swapchain_ctx.swapchain);
        swapchain_ctx.render_pass = create_render_pass(device);
        let (graphics_pipeline, pipeline_layout) =
            create_graphics_pipeline(device, swapchain_ctx.render_pass, surface_caps);
        swapchain_ctx.pipeline_layout = pipeline_layout;
        swapchain_ctx.graphics_pipeline = graphics_pipeline;
        swapchain_ctx.framebuffers =
            create_framebuffers(device, swapchain_ctx.render_pass, &swapchain_ctx.image_views, surface_caps);
    }
}

fn create_swapchain(device: VkDevice, surface: VkSurfaceKHR, surface_caps: VkSurfaceCapabilitiesKHR) -> VkSwapchainKHR {
    unsafe {
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
        swapchain
    }
}

fn create_image_views(device: VkDevice, swapchain: VkSwapchainKHR) -> Vec<VkImageView> {
    unsafe {
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
        swapchain_image_views
    }
}

fn create_render_pass(device: VkDevice) -> VkRenderPass {
    unsafe {
        let mut render_pass = ptr::null_mut();
        check!(vkCreateRenderPass(
            device,
            &VkRenderPassCreateInfo {
                sType: VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                attachmentCount: 1,
                pAttachments: &VkAttachmentDescription {
                    flags: 0,
                    format: VK_FORMAT_B8G8R8A8_SRGB,
                    samples: VK_SAMPLE_COUNT_1_BIT,
                    loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                    storeOp: VK_ATTACHMENT_STORE_OP_STORE,
                    stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                    stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                    initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                    finalLayout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
                },
                subpassCount: 1,
                pSubpasses: &VkSubpassDescription {
                    flags: 0,
                    pipelineBindPoint: VK_PIPELINE_BIND_POINT_GRAPHICS,
                    inputAttachmentCount: 0,
                    pInputAttachments: ptr::null(),
                    colorAttachmentCount: 1,
                    pColorAttachments: &VkAttachmentReference {
                        attachment: 0,
                        layout: VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
                    },
                    pResolveAttachments: ptr::null(),
                    pDepthStencilAttachment: ptr::null(),
                    preserveAttachmentCount: 0,
                    pPreserveAttachments: ptr::null(),
                },
                dependencyCount: 1,
                pDependencies: &VkSubpassDependency {
                    srcSubpass: VK_SUBPASS_EXTERNAL,
                    dstSubpass: 0,
                    srcStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                    dstStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                    srcAccessMask: 0,
                    dstAccessMask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
                    dependencyFlags: 0,
                },
            },
            ptr::null(),
            &mut render_pass,
        ));
        render_pass
    }
}

fn create_graphics_pipeline(
    device: VkDevice,
    render_pass: VkRenderPass,
    surface_caps: VkSurfaceCapabilitiesKHR,
) -> (VkPipeline, VkPipelineLayout) {
    unsafe {
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

        let mut graphics_pipeline = ptr::null_mut();
        check!(vkCreateGraphicsPipelines(
            device,
            ptr::null_mut(),
            1,
            &VkGraphicsPipelineCreateInfo {
                sType: VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                stageCount: 2,
                pStages: [
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
                ]
                .as_ptr(),
                pVertexInputState: &VkPipelineVertexInputStateCreateInfo {
                    sType: VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    vertexBindingDescriptionCount: 0,
                    pVertexBindingDescriptions: ptr::null(),
                    vertexAttributeDescriptionCount: 0,
                    pVertexAttributeDescriptions: ptr::null(),
                },
                pInputAssemblyState: &VkPipelineInputAssemblyStateCreateInfo {
                    sType: VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    topology: VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
                    primitiveRestartEnable: VK_FALSE,
                },
                pTessellationState: ptr::null(),
                pViewportState: &VkPipelineViewportStateCreateInfo {
                    sType: VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    viewportCount: 1,
                    pViewports: &VkViewport {
                        x: 0.0,
                        y: 0.0,
                        width: surface_caps.currentExtent.width as f32,
                        height: surface_caps.currentExtent.height as f32,
                        minDepth: 0.0,
                        maxDepth: 1.0,
                    },
                    scissorCount: 1,
                    pScissors: &VkRect2D {
                        offset: VkOffset2D {
                            x: 0,
                            y: 0,
                        },
                        extent: surface_caps.currentExtent,
                    },
                },
                pRasterizationState: &VkPipelineRasterizationStateCreateInfo {
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
                },
                pMultisampleState: &VkPipelineMultisampleStateCreateInfo {
                    sType: VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    rasterizationSamples: VK_SAMPLE_COUNT_1_BIT,
                    sampleShadingEnable: VK_FALSE,
                    minSampleShading: 1.0,
                    pSampleMask: ptr::null(),
                    alphaToCoverageEnable: VK_FALSE,
                    alphaToOneEnable: VK_FALSE,
                },
                pDepthStencilState: ptr::null(),
                pColorBlendState: &VkPipelineColorBlendStateCreateInfo {
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
                },
                pDynamicState: ptr::null(),
                layout: pipeline_layout,
                renderPass: render_pass,
                subpass: 0,
                basePipelineHandle: ptr::null_mut(),
                basePipelineIndex: -1,
            },
            ptr::null(),
            &mut graphics_pipeline
        ));

        vkDestroyShaderModule(device, fs_shader_module, ptr::null());
        vkDestroyShaderModule(device, vs_shader_module, ptr::null());

        (graphics_pipeline, pipeline_layout)
    }
}

fn create_framebuffers(
    device: VkDevice,
    render_pass: VkRenderPass,
    swapchain_image_views: &[VkImageView],
    surface_caps: VkSurfaceCapabilitiesKHR,
) -> Vec<VkFramebuffer> {
    unsafe {
        let mut framebuffers = vec![ptr::null_mut(); swapchain_image_views.len()];
        for i in 0..swapchain_image_views.len() {
            check!(vkCreateFramebuffer(
                device,
                &VkFramebufferCreateInfo {
                    sType: VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                    pNext: ptr::null(),
                    flags: 0,
                    renderPass: render_pass,
                    attachmentCount: 1,
                    pAttachments: &swapchain_image_views[i],
                    width: surface_caps.currentExtent.width,
                    height: surface_caps.currentExtent.height,
                    layers: 1,
                },
                ptr::null(),
                &mut framebuffers[i]
            ));
        }
        framebuffers
    }
}

fn cleanup_swapchain(device: VkDevice, swapchain_ctx: &mut SwapchainContext) {
    unsafe {
        for framebuffer in &swapchain_ctx.framebuffers {
            vkDestroyFramebuffer(device, *framebuffer, ptr::null());
        }
        vkDestroyPipeline(device, swapchain_ctx.graphics_pipeline, ptr::null());
        vkDestroyRenderPass(device, swapchain_ctx.render_pass, ptr::null());
        vkDestroyPipelineLayout(device, swapchain_ctx.pipeline_layout, ptr::null());

        for image_view in &swapchain_ctx.image_views {
            vkDestroyImageView(device, *image_view, ptr::null());
        }

        vkDestroySwapchainKHR(device, swapchain_ctx.swapchain, ptr::null());
    }
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
