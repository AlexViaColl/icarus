#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
use icarus::*;

use core::ffi::c_void;
use std::ffi::CStr;
use std::fs;
use std::mem;
use std::process;
use std::ptr;
use std::time::Instant;

const BG_COLOR: u64 = 0x00000000;
const MAX_FRAMES_IN_FLIGHT: usize = 2;
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

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

#[repr(C)]
struct Vertex {
    pos: (f32, f32),        // 8
    color: (f32, f32, f32), // 12
}

#[repr(C)]
struct UniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

impl Vertex {
    fn get_binding_description() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Self>() as u32,
            inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    fn get_attribute_descriptions() -> [VkVertexInputAttributeDescription; 2] {
        [
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: VK_FORMAT_R32G32_SFLOAT,
                offset: 0,
            },
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: mem::size_of::<(f32, f32)>() as u32,
            },
        ]
    }
}

fn main() {
    let vertices: Vec<Vertex> = vec![
        Vertex {
            pos: (-0.5, -0.5),
            color: (1.0, 0.0, 0.0),
        },
        Vertex {
            pos: (0.5, -0.5),
            color: (0.0, 1.0, 0.0),
        },
        Vertex {
            pos: (0.5, 0.5),
            color: (0.0, 0.0, 1.0),
        },
        Vertex {
            pos: (-0.5, 0.5),
            color: (1.0, 1.0, 1.0),
        },
    ];
    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

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
        let mut window_width = WINDOW_WIDTH;
        let mut window_height = WINDOW_HEIGHT;
        let window = XCreateSimpleWindow(display, root, 0, 0, window_width, window_height, 1, 0, BG_COLOR);

        assert_ne!(XStoreName(display, window, b"Icarus\0".as_ptr() as *const i8), 0);
        assert_ne!(XSelectInput(display, window, KeyPressMask | ExposureMask | StructureNotifyMask), 0);
        assert_ne!(XMapWindow(display, window), 0);

        // Vulkan initialization
        let mut extension_count = 0;
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, ptr::null_mut()));

        let mut extensions = vec![VkExtensionProperties::default(); extension_count as usize];
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, extensions.as_mut_ptr(),));
        println!("Extensions ({}):", extension_count);
        // for extension in &extensions {
        //     println!("{}", cstr_to_string(extension.extensionName.as_ptr()));
        // }
        // println!();

        let mut layer_count = 0;
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, ptr::null_mut()));
        let mut layers = vec![VkLayerProperties::default(); layer_count as usize];
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr()));
        println!("Layers ({}):", layer_count);
        // for layer in &layers {
        //     println!("{}: {}", cstr_to_string(layer.layerName.as_ptr()), cstr_to_string(layer.description.as_ptr()));
        // }
        // println!();

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
        // println!("Surface: {:?}", surface);

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
            #[allow(unused_variables, unused_mut)]
            for (index, queue_family) in queue_families.iter().enumerate() {
                if (*queue_family).queueFlags & VK_QUEUE_GRAPHICS_BIT != 0 {
                    // println!("Found a queue {} with VK_QUEUE_GRAPHICS_BIT", index);
                }
                let mut present_support = 0;
                //vkGetPhysicalDeviceSurfaceSupportKHR(*device, index as u32, surface, &mut present_support);
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
            // println!("Queue supports presentation and graphics operations.");
        } else {
            panic!("Queue doesn't support presentation!");
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
        // println!("{:#?}", surface_formats);

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
        // println!("{:#?}", surface_present_modes);

        assert!(surface_formats.contains(&VkSurfaceFormatKHR {
            format: VK_FORMAT_B8G8R8A8_SRGB,
            colorSpace: VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
        }));
        assert!(surface_present_modes.contains(&VK_PRESENT_MODE_MAILBOX_KHR));

        let swapchain = create_swapchain(device, surface, surface_caps);
        let swapchain_image_views = create_image_views(device, swapchain);

        let mut descriptor_set_layout = ptr::null_mut();
        check!(vkCreateDescriptorSetLayout(
            device,
            &VkDescriptorSetLayoutCreateInfo {
                sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                bindingCount: 1,
                pBindings: &VkDescriptorSetLayoutBinding {
                    binding: 0,
                    descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                    descriptorCount: 1,
                    stageFlags: VK_SHADER_STAGE_VERTEX_BIT,
                    pImmutableSamplers: ptr::null(),
                },
            },
            ptr::null(),
            &mut descriptor_set_layout
        ));

        let render_pass = create_render_pass(device);
        let (graphics_pipeline, pipeline_layout) =
            create_graphics_pipeline(device, render_pass, surface_caps, descriptor_set_layout);
        let framebuffers = create_framebuffers(device, render_pass, &swapchain_image_views, surface_caps);
        let descriptor_pool = create_descriptor_pool(device);

        let mut descriptor_sets = vec![ptr::null_mut(); MAX_FRAMES_IN_FLIGHT];
        check!(vkAllocateDescriptorSets(
            device,
            &VkDescriptorSetAllocateInfo {
                sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
                pNext: ptr::null(),
                descriptorPool: descriptor_pool,
                descriptorSetCount: MAX_FRAMES_IN_FLIGHT as u32,
                pSetLayouts: vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT].as_ptr(),
            },
            descriptor_sets.as_mut_ptr()
        ));

        let mut swapchain_ctx = SwapchainContext {
            physical_device,
            swapchain,
            image_views: swapchain_image_views,
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layout,
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

        let (texture_image, texture_image_memory) =
            create_texture_image(device, physical_device, graphics_queue, command_pool);

        let (vertex_buffer, vertex_buffer_memory) =
            create_vertex_buffer(device, physical_device, graphics_queue, command_pool, &vertices);
        let (index_buffer, index_buffer_memory) =
            create_index_buffer(device, physical_device, graphics_queue, command_pool, &indices);

        // Create Uniform Buffers
        let mut uniform_buffers = vec![];
        let mut uniform_buffers_memory = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (uniform_buffer, uniform_buffer_memory) = create_buffer(
                device,
                physical_device,
                mem::size_of::<UniformBufferObject>() as VkDeviceSize,
                VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
                VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
            );
            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vkUpdateDescriptorSets(
                device,
                1,
                &VkWriteDescriptorSet {
                    sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
                    pNext: ptr::null(),
                    dstSet: swapchain_ctx.descriptor_sets[i],
                    dstBinding: 0,
                    dstArrayElement: 0,
                    descriptorCount: 1,
                    descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                    pImageInfo: ptr::null(),
                    pBufferInfo: &VkDescriptorBufferInfo {
                        buffer: uniform_buffers[i],
                        offset: 0,
                        range: mem::size_of::<UniformBufferObject>() as VkDeviceSize,
                    },
                    pTexelBufferView: ptr::null(),
                },
                0,
                ptr::null(),
            );
        }

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
        let mut framebuffer_resized = false;
        let start_time = Instant::now();
        while running {
            while XPending(display) > 0 {
                let mut event = XEvent {
                    pad: [0; 24],
                };
                XNextEvent(display, &mut event);
                match event.ttype {
                    KeyPress => {
                        #[allow(unused_variables)]
                        let keysym = XLookupKeysym(&mut event.xkey, 0);
                        let event = event.xkey;
                        // println!("KeySym: {} / KeyCode: {}", keysym, event.keycode);
                        match event.keycode {
                            9 => running = false,
                            //n => println!("Keycode: {}", n),
                            _ => {}
                        }
                    }
                    Expose => {
                        // let gc = XDefaultGC(display, screen);
                        // XFillRectangle(display, window, gc, 20, 20, 10, 10);
                    }
                    ConfigureNotify => {
                        let event = event.xconfigure;
                        if event.width as u32 != window_width || event.height as u32 != window_height {
                            window_width = event.width as u32;
                            window_height = event.height as u32;
                            // println!("ConfigureNotify ({}, {})", window_width, window_height);
                            //framebuffer_resized = true;
                            check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
                                physical_device,
                                surface,
                                &mut surface_caps
                            ));
                        }
                    }
                    _ => {}
                }
            }

            // draw
            check!(vkWaitForFences(device, 1, &in_flight_fences[current_frame], VK_TRUE, u64::MAX));

            let mut image_index = 0;
            let recreate = match vkAcquireNextImageKHR(
                device,
                swapchain_ctx.swapchain,
                u64::MAX,
                image_available_semaphores[current_frame],
                ptr::null_mut(),
                &mut image_index,
            ) {
                VK_SUCCESS | VK_SUBOPTIMAL_KHR => false,
                VK_ERROR_OUT_OF_DATE_KHR => true,
                res => panic!("{:?}", res),
            };
            if recreate {
                recreate_swapchain(&mut swapchain_ctx, device, surface, &uniform_buffers);
                continue;
            }

            // Update the uniforms
            let seconds_elapsed = start_time.elapsed().as_secs_f32();
            let ubo = UniformBufferObject {
                proj: Mat4::identity(),
                view: Mat4::identity(),
                model: Mat4::rotate(seconds_elapsed * std::f32::consts::PI / 4.0, (0.0, 0.0, 1.0)),
            };
            let mut data = ptr::null_mut();
            vkMapMemory(
                device,
                uniform_buffers_memory[current_frame],
                0,
                mem::size_of::<UniformBufferObject>() as VkDeviceSize,
                0,
                &mut data,
            );
            std::ptr::copy(&ubo, data as *mut UniformBufferObject, 1);
            vkUnmapMemory(device, uniform_buffers_memory[current_frame]);

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

            vkCmdBindVertexBuffers(command_buffers[current_frame], 0, 1, &vertex_buffer, &0);
            vkCmdBindIndexBuffer(command_buffers[current_frame], index_buffer, 0, VK_INDEX_TYPE_UINT16);

            vkCmdBindDescriptorSets(
                command_buffers[current_frame],
                VK_PIPELINE_BIND_POINT_GRAPHICS,
                swapchain_ctx.pipeline_layout,
                0,
                1,
                &swapchain_ctx.descriptor_sets[current_frame],
                0,
                ptr::null(),
            );
            // vkCmdDraw(command_buffers[current_frame], vertices.len() as u32, 1, 0, 0);
            vkCmdDrawIndexed(command_buffers[current_frame], indices.len() as u32, 1, 0, 0, 0);

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

            let recreate = match vkQueuePresentKHR(
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
                VK_SUCCESS => false,
                VK_SUBOPTIMAL_KHR | VK_ERROR_OUT_OF_DATE_KHR => true,
                res => panic!("{:?}", res),
            };
            if recreate || framebuffer_resized {
                recreate_swapchain(&mut swapchain_ctx, device, surface, &uniform_buffers);
                framebuffer_resized = false;
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

        vkDestroyDescriptorPool(device, swapchain_ctx.descriptor_pool, ptr::null());
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vkFreeMemory(device, uniform_buffers_memory[i], ptr::null());
            vkDestroyBuffer(device, uniform_buffers[i], ptr::null());
        }

        vkFreeMemory(device, texture_image_memory, ptr::null());
        vkDestroyImage(device, texture_image, ptr::null());

        vkFreeMemory(device, index_buffer_memory, ptr::null());
        vkDestroyBuffer(device, index_buffer, ptr::null());
        vkFreeMemory(device, vertex_buffer_memory, ptr::null());
        vkDestroyBuffer(device, vertex_buffer, ptr::null());
        vkDestroyCommandPool(device, command_pool, ptr::null());
        cleanup_swapchain(device, &mut swapchain_ctx);
        vkDestroyDescriptorSetLayout(device, descriptor_set_layout, ptr::null());

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
    physical_device: VkPhysicalDevice,
    swapchain: VkSwapchainKHR,
    image_views: Vec<VkImageView>,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: Vec<VkDescriptorSet>,
    descriptor_set_layout: VkDescriptorSetLayout,
    render_pass: VkRenderPass,
    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,
    framebuffers: Vec<VkFramebuffer>,
}

fn recreate_swapchain(
    swapchain_ctx: &mut SwapchainContext,
    device: VkDevice,
    surface: VkSurfaceKHR,
    uniform_buffers: &[VkBuffer],
) {
    unsafe {
        vkDeviceWaitIdle(device);

        cleanup_swapchain(device, swapchain_ctx);

        let mut surface_caps = VkSurfaceCapabilitiesKHR::default();
        check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(swapchain_ctx.physical_device, surface, &mut surface_caps));

        swapchain_ctx.swapchain = create_swapchain(device, surface, surface_caps);
        swapchain_ctx.image_views = create_image_views(device, swapchain_ctx.swapchain);
        swapchain_ctx.render_pass = create_render_pass(device);
        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(
            device,
            swapchain_ctx.render_pass,
            surface_caps,
            swapchain_ctx.descriptor_set_layout,
        );
        swapchain_ctx.pipeline_layout = pipeline_layout;
        swapchain_ctx.graphics_pipeline = graphics_pipeline;
        swapchain_ctx.framebuffers =
            create_framebuffers(device, swapchain_ctx.render_pass, &swapchain_ctx.image_views, surface_caps);

        vkDestroyDescriptorPool(device, swapchain_ctx.descriptor_pool, ptr::null());
        swapchain_ctx.descriptor_pool = create_descriptor_pool(device);
        check!(vkAllocateDescriptorSets(
            device,
            &VkDescriptorSetAllocateInfo {
                sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
                pNext: ptr::null(),
                descriptorPool: swapchain_ctx.descriptor_pool,
                descriptorSetCount: MAX_FRAMES_IN_FLIGHT as u32,
                pSetLayouts: vec![swapchain_ctx.descriptor_set_layout; MAX_FRAMES_IN_FLIGHT].as_ptr(),
            },
            swapchain_ctx.descriptor_sets.as_mut_ptr()
        ));

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vkUpdateDescriptorSets(
                device,
                1,
                &VkWriteDescriptorSet {
                    sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
                    pNext: ptr::null(),
                    dstSet: swapchain_ctx.descriptor_sets[i],
                    dstBinding: 0,
                    dstArrayElement: 0,
                    descriptorCount: 1,
                    descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                    pImageInfo: ptr::null(),
                    pBufferInfo: &VkDescriptorBufferInfo {
                        buffer: uniform_buffers[i],
                        offset: 0,
                        range: mem::size_of::<UniformBufferObject>() as VkDeviceSize,
                    },
                    pTexelBufferView: ptr::null(),
                },
                0,
                ptr::null(),
            );
        }
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
        // println!("Swapchain created with {} images", swapchain_image_count);

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
    descriptor_set_layout: VkDescriptorSetLayout,
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
                setLayoutCount: 1,
                pSetLayouts: &descriptor_set_layout,
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
                    vertexBindingDescriptionCount: 1,
                    pVertexBindingDescriptions: &Vertex::get_binding_description(),
                    vertexAttributeDescriptionCount: Vertex::get_attribute_descriptions().len() as u32,
                    pVertexAttributeDescriptions: Vertex::get_attribute_descriptions().as_ptr(),
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

fn create_descriptor_pool(device: VkDevice) -> VkDescriptorPool {
    unsafe {
        let mut descriptor_pool = ptr::null_mut();
        check!(vkCreateDescriptorPool(
            device,
            &VkDescriptorPoolCreateInfo {
                sType: VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                maxSets: MAX_FRAMES_IN_FLIGHT as u32,
                poolSizeCount: 1,
                pPoolSizes: &VkDescriptorPoolSize {
                    ttype: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                    descriptorCount: MAX_FRAMES_IN_FLIGHT as u32,
                }
            },
            ptr::null(),
            &mut descriptor_pool
        ));
        descriptor_pool
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

fn find_memory_type(physical_device: VkPhysicalDevice, type_filter: u32, properties: VkMemoryPropertyFlags) -> u32 {
    let mut mem_properties = VkPhysicalDeviceMemoryProperties::default();
    unsafe { vkGetPhysicalDeviceMemoryProperties(physical_device, &mut mem_properties) };

    for i in 0..mem_properties.memoryTypeCount {
        if type_filter & (1 << i) != 0
            && mem_properties.memoryTypes[i as usize].propertyFlags & properties == properties
        {
            return i;
        }
    }

    panic!("Failed to find suitable memory type!");
}

fn copy_buffer(
    device: VkDevice,
    command_pool: VkCommandPool,
    graphics_queue: VkQueue,
    src_buffer: VkBuffer,
    dst_buffer: VkBuffer,
    size: VkDeviceSize,
) {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool);

        vkCmdCopyBuffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            1,
            &VkBufferCopy {
                srcOffset: 0,
                dstOffset: 0,
                size,
            },
        );

        end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
    }
}

fn create_vertex_buffer(
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    vertices: &[Vertex],
) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let buffer_size = (mem::size_of_val(&vertices[0]) * vertices.len()) as VkDeviceSize;
        let (staging_buffer, staging_buffer_memory) = create_buffer(
            device,
            physical_device,
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
        );

        let mut data = ptr::null_mut();
        vkMapMemory(device, staging_buffer_memory, 0, buffer_size, 0, &mut data);
        std::ptr::copy(vertices.as_ptr(), data as *mut Vertex, vertices.len());
        vkUnmapMemory(device, staging_buffer_memory);

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            device,
            physical_device,
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
        );

        copy_buffer(device, command_pool, graphics_queue, staging_buffer, vertex_buffer, buffer_size);

        vkFreeMemory(device, staging_buffer_memory, ptr::null());
        vkDestroyBuffer(device, staging_buffer, ptr::null());

        (vertex_buffer, vertex_buffer_memory)
    }
}

fn create_index_buffer(
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    indices: &[u16],
) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let buffer_size = (mem::size_of_val(&indices[0]) * indices.len()) as VkDeviceSize;
        let (staging_buffer, staging_buffer_memory) = create_buffer(
            device,
            physical_device,
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
        );

        let mut data = ptr::null_mut();
        vkMapMemory(device, staging_buffer_memory, 0, buffer_size, 0, &mut data);
        std::ptr::copy(indices.as_ptr(), data as *mut u16, indices.len());
        vkUnmapMemory(device, staging_buffer_memory);

        let (index_buffer, index_buffer_memory) = create_buffer(
            device,
            physical_device,
            buffer_size,
            VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
        );

        copy_buffer(device, command_pool, graphics_queue, staging_buffer, index_buffer, buffer_size);

        vkFreeMemory(device, staging_buffer_memory, ptr::null());
        vkDestroyBuffer(device, staging_buffer, ptr::null());

        (index_buffer, index_buffer_memory)
    }
}

fn create_buffer(
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    size: VkDeviceSize,
    usage: VkBufferUsageFlags,
    properties: VkMemoryPropertyFlags,
) -> (VkBuffer, VkDeviceMemory) {
    unsafe {
        let mut buffer = ptr::null_mut();
        check!(vkCreateBuffer(
            device,
            &VkBufferCreateInfo {
                sType: VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                size,
                usage,
                sharingMode: VK_SHARING_MODE_EXCLUSIVE,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
            },
            ptr::null(),
            &mut buffer
        ));
        let mut mem_requirements = VkMemoryRequirements::default();
        vkGetBufferMemoryRequirements(device, buffer, &mut mem_requirements);

        let mut buffer_memory = ptr::null_mut();
        check!(vkAllocateMemory(
            device,
            &VkMemoryAllocateInfo {
                sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                pNext: ptr::null(),
                allocationSize: mem_requirements.size,
                memoryTypeIndex: find_memory_type(physical_device, mem_requirements.memoryTypeBits, properties),
            },
            ptr::null(),
            &mut buffer_memory
        ));

        check!(vkBindBufferMemory(device, buffer, buffer_memory, 0));

        (buffer, buffer_memory)
    }
}

fn create_texture_image(
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
) -> (VkImage, VkDeviceMemory) {
    unsafe {
        let mut width = 0;
        let mut height = 0;
        let mut channels = 0;
        let pixels =
            stbi_load(b"textures/texture.jpg\0".as_ptr() as *const i8, &mut width, &mut height, &mut channels, 4);
        assert!(!pixels.is_null());
        let image_size = width * height * 4;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            device,
            physical_device,
            image_size as VkDeviceSize,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
        );
        let mut data = ptr::null_mut();
        vkMapMemory(device, staging_buffer_memory, 0, image_size as VkDeviceSize, 0, &mut data);
        std::ptr::copy(pixels, data as *mut u8, image_size as usize);
        vkUnmapMemory(device, staging_buffer_memory);

        stbi_image_free(pixels as *mut c_void);

        let mut texture_image = ptr::null_mut();
        check!(vkCreateImage(
            device,
            &VkImageCreateInfo {
                sType: VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                imageType: VK_IMAGE_TYPE_2D,
                format: VK_FORMAT_R8G8B8A8_SRGB,
                extent: VkExtent3D {
                    width: width as u32,
                    height: height as u32,
                    depth: 1,
                },
                mipLevels: 1,
                arrayLayers: 1,
                samples: VK_SAMPLE_COUNT_1_BIT, // VkSampleCountFlagBits
                tiling: VK_IMAGE_TILING_OPTIMAL,
                usage: VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT,
                sharingMode: VK_SHARING_MODE_EXCLUSIVE,
                queueFamilyIndexCount: 0,
                pQueueFamilyIndices: ptr::null(),
                initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
            },
            ptr::null(),
            &mut texture_image
        ));

        let mut memory_requirements = VkMemoryRequirements::default();
        vkGetImageMemoryRequirements(device, texture_image, &mut memory_requirements);

        let mut texture_image_memory = ptr::null_mut();
        check!(vkAllocateMemory(
            device,
            &VkMemoryAllocateInfo {
                sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
                pNext: ptr::null(),
                allocationSize: memory_requirements.size,
                memoryTypeIndex: find_memory_type(
                    physical_device,
                    memory_requirements.memoryTypeBits,
                    VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT
                ),
            },
            ptr::null(),
            &mut texture_image_memory
        ));

        check!(vkBindImageMemory(device, texture_image, texture_image_memory, 0));

        transition_image_layout(
            device,
            graphics_queue,
            command_pool,
            texture_image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_UNDEFINED,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        );

        copy_buffer_to_image(
            device,
            graphics_queue,
            command_pool,
            staging_buffer,
            texture_image,
            width as u32,
            height as u32,
        );

        transition_image_layout(
            device,
            graphics_queue,
            command_pool,
            texture_image,
            VK_FORMAT_R8G8B8A8_SRGB,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        );

        vkFreeMemory(device, staging_buffer_memory, ptr::null());
        vkDestroyBuffer(device, staging_buffer, ptr::null());

        (texture_image, texture_image_memory)
    }
}

fn copy_buffer_to_image(
    device: VkDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    buffer: VkBuffer,
    image: VkImage,
    width: u32,
    height: u32,
) {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool);
        vkCmdCopyBufferToImage(
            command_buffer,
            buffer,
            image,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            1,
            &VkBufferImageCopy {
                bufferOffset: 0,
                bufferRowLength: 0,
                bufferImageHeight: 0,
                imageSubresource: VkImageSubresourceLayers {
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                    mipLevel: 0,
                    baseArrayLayer: 0,
                    layerCount: 1,
                },
                imageOffset: VkOffset3D {
                    x: 0,
                    y: 0,
                    z: 0,
                },
                imageExtent: VkExtent3D {
                    width,
                    height,
                    depth: 1,
                },
            },
        );
        end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
    }
}

fn transition_image_layout(
    device: VkDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    image: VkImage,
    _format: VkFormat,
    old_layout: VkImageLayout,
    new_layout: VkImageLayout,
) {
    unsafe {
        let command_buffer = begin_single_time_commands(device, command_pool);
        let (src_access_mask, dst_access_mask, src_stage, dst_stage) =
            if old_layout == VK_IMAGE_LAYOUT_UNDEFINED && new_layout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL {
                (
                    VK_ACCESS_NONE,
                    VK_ACCESS_TRANSFER_WRITE_BIT,
                    VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                )
            } else if old_layout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
                && new_layout == VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
            {
                (
                    VK_ACCESS_TRANSFER_WRITE_BIT,
                    VK_ACCESS_SHADER_READ_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
                )
            } else {
                panic!("Invalid combination of old_layout: {:?} / new_layout: {:?}", old_layout, new_layout);
            };

        vkCmdPipelineBarrier(
            command_buffer,
            src_stage,
            dst_stage,
            0,
            0,
            ptr::null(),
            0,
            ptr::null(),
            1,
            &VkImageMemoryBarrier {
                sType: VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
                pNext: ptr::null(),
                srcAccessMask: src_access_mask,
                dstAccessMask: dst_access_mask,
                oldLayout: old_layout,
                newLayout: new_layout,
                srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                image: image,
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                    baseMipLevel: 0,
                    levelCount: 1,
                    baseArrayLayer: 0,
                    layerCount: 1,
                },
            },
        );
        end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
    }
}

fn begin_single_time_commands(device: VkDevice, command_pool: VkCommandPool) -> VkCommandBuffer {
    unsafe {
        let mut command_buffer = ptr::null_mut();
        check!(vkAllocateCommandBuffers(
            device,
            &VkCommandBufferAllocateInfo {
                sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
                pNext: ptr::null(),
                commandPool: command_pool,
                level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
                commandBufferCount: 1,
            },
            &mut command_buffer
        ));

        check!(vkBeginCommandBuffer(
            command_buffer,
            &VkCommandBufferBeginInfo {
                sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
                pNext: ptr::null(),
                flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
                pInheritanceInfo: ptr::null(),
            }
        ));

        command_buffer
    }
}

fn end_single_time_commands(
    device: VkDevice,
    graphics_queue: VkQueue,
    command_pool: VkCommandPool,
    command_buffer: VkCommandBuffer,
) {
    unsafe {
        check!(vkEndCommandBuffer(command_buffer));

        check!(vkQueueSubmit(
            graphics_queue,
            1,
            &VkSubmitInfo {
                sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: ptr::null(),
                waitSemaphoreCount: 0,
                pWaitSemaphores: ptr::null(),
                pWaitDstStageMask: ptr::null(),
                commandBufferCount: 1,
                pCommandBuffers: &command_buffer,
                signalSemaphoreCount: 0,
                pSignalSemaphores: ptr::null(),
            },
            ptr::null_mut()
        ));

        check!(vkQueueWaitIdle(graphics_queue));

        vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);
    }
}

extern "C" fn debug_callback(
    _message_severity: VkDebugUtilsMessageSeverityFlagsEXT,
    _message_type: VkDebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const VkDebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> VkBool32 {
    unsafe {
        // TODO: Possible false positive validation errors:
        // https://github.com/KhronosGroup/Vulkan-ValidationLayers/issues/1340

        //  2094043421 VUID-VkSwapchainCreateInfoKHR-imageExtent-01274
        // -1615083365 VUID-VkRenderPassBeginInfo-renderArea-02848
        // -1280461305 VUID-VkRenderPassBeginInfo-renderArea-02849
        match (*p_callback_data).messageIdNumber {
            2094043421 => {}
            -1615083365 => {}
            -1280461305 => {}
            _ => {
                println!(
                    "{} {}",
                    (*p_callback_data).messageIdNumber,
                    CStr::from_ptr((*p_callback_data).pMessage).to_string_lossy()
                );
            }
        }
        0
    }
}
