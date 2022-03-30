use super::string_util::*;
use super::vk::*;

use crate::cstr;

use core::ffi::c_void;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::ptr;

#[macro_export]
macro_rules! check(
    ($expression:expr) => {
        assert_eq!($expression, VK_SUCCESS);
    }
);

pub fn vk_version_to_string(version: u32) -> String {
    format!(
        //"{}.{}.{}.{}",
        "{}.{}.{}",
        //VK_API_VERSION_VARIANT(version),
        VK_API_VERSION_MAJOR(version),
        VK_API_VERSION_MINOR(version),
        VK_API_VERSION_PATCH(version)
    )
}

pub fn vk_enumerate_instance_extension_properties() -> Vec<VkExtensionProperties> {
    unsafe {
        let mut extension_count = 0;
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, ptr::null_mut()));
        let mut extensions = vec![VkExtensionProperties::default(); extension_count as usize];
        check!(vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, extensions.as_mut_ptr(),));
        extensions
    }
}

pub fn vk_enumerate_instance_layer_properties() -> Vec<VkLayerProperties> {
    unsafe {
        let mut layer_count = 0;
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, ptr::null_mut()));
        let mut layers = vec![VkLayerProperties::default(); layer_count as usize];
        check!(vkEnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr()));
        layers
    }
}

pub fn vk_create_instance(layers: &[*const i8], extensions: &[*const i8]) -> VkInstance {
    unsafe {
        let mut instance = VkInstance::default();
        check!(vkCreateInstance(
            &VkInstanceCreateInfo {
                pApplicationInfo: &VkApplicationInfo {
                    ..VkApplicationInfo::default()
                },
                enabledLayerCount: layers.len() as u32,
                ppEnabledLayerNames: layers.as_ptr(),
                enabledExtensionCount: extensions.len() as u32,
                ppEnabledExtensionNames: extensions.as_ptr(),
                ..VkInstanceCreateInfo::default()
            },
            ptr::null(),
            &mut instance,
        ));
        instance
    }
}

pub fn vk_enumerate_physical_devices(instance: VkInstance) -> Vec<VkPhysicalDevice> {
    unsafe {
        let mut device_count = 0;
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, ptr::null_mut()));
        let mut physical_devices = vec![VkPhysicalDevice::default(); device_count as usize];
        check!(vkEnumeratePhysicalDevices(instance, &mut device_count, physical_devices.as_mut_ptr()));
        physical_devices
    }
}

pub fn vk_get_physical_device_properties(physical_device: VkPhysicalDevice) -> VkPhysicalDeviceProperties {
    unsafe {
        let mut properties = VkPhysicalDeviceProperties::default();
        vkGetPhysicalDeviceProperties(physical_device, &mut properties);
        properties
    }
}

pub fn vk_get_physical_device_features(physical_device: VkPhysicalDevice) -> VkPhysicalDeviceFeatures {
    unsafe {
        let mut features = VkPhysicalDeviceFeatures::default();
        vkGetPhysicalDeviceFeatures(physical_device, &mut features);
        features
    }
}

pub fn vk_enumerate_device_extension_properties(physical_device: VkPhysicalDevice) -> Vec<VkExtensionProperties> {
    unsafe {
        let mut extension_count = 0;
        check!(vkEnumerateDeviceExtensionProperties(
            physical_device,
            ptr::null(),
            &mut extension_count,
            ptr::null_mut()
        ));
        let mut extensions = vec![VkExtensionProperties::default(); extension_count as usize];
        check!(vkEnumerateDeviceExtensionProperties(
            physical_device,
            ptr::null(),
            &mut extension_count,
            extensions.as_mut_ptr(),
        ));
        extensions
    }
}

pub fn vk_get_physical_device_surface_support_khr(
    physical_device: VkPhysicalDevice,
    queue_family_index: u32,
    surface: VkSurfaceKHR,
) -> VkBool32 {
    unsafe {
        let mut present_support = VK_FALSE;
        vkGetPhysicalDeviceSurfaceSupportKHR(physical_device, queue_family_index, surface, &mut present_support);
        present_support
    }
}

pub fn vk_get_physical_device_surface_capabilities_khr(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> VkSurfaceCapabilitiesKHR {
    unsafe {
        let mut surface_caps = VkSurfaceCapabilitiesKHR::default();
        check!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface, &mut surface_caps));
        surface_caps
    }
}

pub fn vk_get_physical_device_memory_properties(physical_device: VkPhysicalDevice) -> VkPhysicalDeviceMemoryProperties {
    unsafe {
        let mut mem_props = VkPhysicalDeviceMemoryProperties::default();
        vkGetPhysicalDeviceMemoryProperties(physical_device, &mut mem_props);
        mem_props
    }
}

pub fn vk_get_physical_device_format_properties(
    physical_device: VkPhysicalDevice,
    format: VkFormat,
) -> VkFormatProperties {
    unsafe {
        let mut format_props = VkFormatProperties::default();
        vkGetPhysicalDeviceFormatProperties(physical_device, format, &mut format_props);
        format_props
    }
}

pub fn vk_create_debug_utils_messenger_ext(
    instance: VkInstance,
    debug_callback: PFN_vkDebugUtilsMessengerCallbackEXT,
) -> VkDebugUtilsMessengerEXT {
    unsafe {
        let mut debug_messenger = VkDebugUtilsMessengerEXT::default();
        #[allow(non_snake_case)]
        let vkCreateDebugUtilsMessengerEXT = mem::transmute::<_, PFN_vkCreateDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, cstr!("vkCreateDebugUtilsMessengerEXT")),
        );
        check!(vkCreateDebugUtilsMessengerEXT(
            instance,
            #[allow(clippy::identity_op)]
            &VkDebugUtilsMessengerCreateInfoEXT {
                messageSeverity: (0
                    //| VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                    //| VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT)
                    .into(),
                messageType: (VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                    | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT)
                    .into(),
                pfnUserCallback: Some(debug_callback),
                ..VkDebugUtilsMessengerCreateInfoEXT::default()
            },
            ptr::null(),
            &mut debug_messenger,
        ));
        debug_messenger
    }
}

pub fn vk_destroy_debug_utils_messenger_ext(instance: VkInstance, messenger: VkDebugUtilsMessengerEXT) {
    unsafe {
        #[allow(non_snake_case)]
        let vkDestroyDebugUtilsMessengerEXT = std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance, cstr!("vkDestroyDebugUtilsMessengerEXT") as *const i8),
        );
        vkDestroyDebugUtilsMessengerEXT(instance, messenger, ptr::null());
    }
}

pub fn vk_create_xlib_surface_khr(
    instance: VkInstance,
    dpy: *mut crate::x11::Display,
    window: crate::x11::Window,
) -> VkSurfaceKHR {
    unsafe {
        let mut surface = VkSurfaceKHR::default();
        check!(vkCreateXlibSurfaceKHR(
            instance,
            &VkXlibSurfaceCreateInfoKHR {
                dpy,
                window,
                ..VkXlibSurfaceCreateInfoKHR::default()
            },
            ptr::null(),
            &mut surface,
        ));
        surface
    }
}

pub fn vk_create_swapchain_khr(
    device: VkDevice,
    surface: VkSurfaceKHR,
    surface_caps: VkSurfaceCapabilitiesKHR,
    surface_format: VkSurfaceFormatKHR,
) -> VkSwapchainKHR {
    unsafe {
        let mut swapchain = VkSwapchainKHR::default();
        check!(vkCreateSwapchainKHR(
            device,
            &VkSwapchainCreateInfoKHR {
                surface,
                minImageCount: surface_caps.minImageCount + 1,
                imageFormat: surface_format.format,
                imageColorSpace: surface_format.colorSpace,
                imageExtent: surface_caps.currentExtent,
                imageArrayLayers: 1,
                imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT.into(),
                preTransform: surface_caps.currentTransform,
                compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR.into(),
                presentMode: VK_PRESENT_MODE_FIFO_KHR,
                clipped: VK_TRUE,
                ..VkSwapchainCreateInfoKHR::default()
            },
            ptr::null(),
            &mut swapchain
        ));
        swapchain
    }
}

pub fn vk_create_image_view(
    device: VkDevice,
    image: VkImage,
    format: VkFormat,
    aspect: VkImageAspectFlags,
) -> VkImageView {
    unsafe {
        let mut image_view = VkImageView::default();
        check!(vkCreateImageView(
            device,
            &VkImageViewCreateInfo {
                image,
                viewType: VK_IMAGE_VIEW_TYPE_2D,
                format,
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: aspect,
                    levelCount: 1,
                    layerCount: 1,
                    ..VkImageSubresourceRange::default()
                },
                ..VkImageViewCreateInfo::default()
            },
            ptr::null(),
            &mut image_view
        ));
        image_view
    }
}

pub fn vk_get_swapchain_images_khr(device: VkDevice, swapchain: VkSwapchainKHR) -> Vec<VkImage> {
    unsafe {
        let mut swapchain_image_count = 0;
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, ptr::null_mut()));
        let mut swapchain_images = vec![VkImage::default(); swapchain_image_count as usize];
        check!(vkGetSwapchainImagesKHR(device, swapchain, &mut swapchain_image_count, swapchain_images.as_mut_ptr()));
        swapchain_images
    }
}

pub fn vk_get_physical_device_queue_family_properties(
    physical_device: VkPhysicalDevice,
) -> Vec<VkQueueFamilyProperties> {
    unsafe {
        let mut queue_family_count = 0;
        vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, ptr::null_mut());
        let mut queue_families = vec![VkQueueFamilyProperties::default(); queue_family_count as usize];
        vkGetPhysicalDeviceQueueFamilyProperties(physical_device, &mut queue_family_count, queue_families.as_mut_ptr());
        queue_families
    }
}

pub fn vk_get_physical_device_surface_formats_khr(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> Vec<VkSurfaceFormatKHR> {
    unsafe {
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
        surface_formats
    }
}

pub fn vk_get_physical_device_surface_present_modes_khr(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> Vec<VkPresentModeKHR> {
    unsafe {
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
        surface_present_modes
    }
}

pub fn begin_single_time_commands(device: VkDevice, command_pool: VkCommandPool) -> VkCommandBuffer {
    unsafe {
        let mut command_buffer = VkCommandBuffer::default();
        check!(vkAllocateCommandBuffers(
            device,
            &VkCommandBufferAllocateInfo {
                commandPool: command_pool,
                commandBufferCount: 1,
                ..VkCommandBufferAllocateInfo::default()
            },
            &mut command_buffer
        ));

        check!(vkBeginCommandBuffer(
            command_buffer,
            &VkCommandBufferBeginInfo {
                flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT.into(),
                ..VkCommandBufferBeginInfo::default()
            }
        ));

        command_buffer
    }
}

pub fn end_single_time_commands(
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
                commandBufferCount: 1,
                pCommandBuffers: &command_buffer,
                ..VkSubmitInfo::default()
            },
            VkFence::default(),
        ));

        check!(vkQueueWaitIdle(graphics_queue));

        vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);
    }
}

pub fn transition_image_layout(
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
            src_stage.into(),
            dst_stage.into(),
            0.into(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            1,
            &VkImageMemoryBarrier {
                srcAccessMask: src_access_mask.into(),
                dstAccessMask: dst_access_mask.into(),
                oldLayout: old_layout,
                newLayout: new_layout,
                srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
                image,
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT.into(),
                    levelCount: 1,
                    layerCount: 1,
                    ..VkImageSubresourceRange::default()
                },
                ..VkImageMemoryBarrier::default()
            },
        );
        end_single_time_commands(device, graphics_queue, command_pool, command_buffer);
    }
}

pub fn copy_buffer_to_image(
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
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT.into(),
                    mipLevel: 0,
                    baseArrayLayer: 0,
                    layerCount: 1,
                },
                imageOffset: VkOffset3D::default(),
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

pub fn vk_map_memory_copy<T>(device: VkDevice, memory: VkDeviceMemory, data: *const T, size: usize) {
    unsafe {
        let mut mapped = ptr::null_mut();
        vkMapMemory(device, memory, 0, size as VkDeviceSize, 0, &mut mapped);
        ptr::copy(data as *const u8, mapped as *mut u8, size);
        vkUnmapMemory(device, memory);
    }
}

#[derive(Default)]
pub struct Buffer {
    pub device: VkDevice,
    pub buffer: VkBuffer,
    pub memory: VkDeviceMemory,
}
impl Drop for Buffer {
    fn drop(&mut self) {
        if self.device != VkDevice::default() {
            unsafe { vkFreeMemory(self.device, self.memory, ptr::null()) };
            unsafe { vkDestroyBuffer(self.device, self.buffer, ptr::null()) };
        }
    }
}

#[derive(Default)]
pub struct Image {
    pub device: VkDevice,
    pub image: VkImage,
    pub memory: VkDeviceMemory,
    pub view: VkImageView,
}
impl Drop for Image {
    fn drop(&mut self) {
        if self.device != VkDevice::default() {
            unsafe { vkDestroyImageView(self.device, self.view, ptr::null()) };
            unsafe { vkFreeMemory(self.device, self.memory, ptr::null()) };
            unsafe { vkDestroyImage(self.device, self.image, ptr::null()) };
        }
    }
}

impl VkRect2D {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            offset: VkOffset2D {
                x,
                y,
            },
            extent: VkExtent2D {
                width,
                height,
            },
        }
    }
}

impl VkViewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            minDepth: min_z,
            maxDepth: max_z,
        }
    }
}

impl VkClearColorValue {
    pub fn new(value: [f32; 4]) -> VkClearValue {
        VkClearValue {
            color: VkClearColorValue {
                float32: value,
            },
        }
    }
}

impl VkClearDepthStencilValue {
    pub fn new(depth: f32, stencil: u32) -> VkClearValue {
        VkClearValue {
            depthStencil: VkClearDepthStencilValue {
                depth,
                stencil,
            },
        }
    }
}

impl VkBufferCopy {
    pub fn new(src_offset: usize, dst_offset: usize, size: usize) -> Self {
        Self {
            srcOffset: src_offset as VkDeviceSize,
            dstOffset: dst_offset as VkDeviceSize,
            size: size as VkDeviceSize,
        }
    }
}

pub fn layout_binding(
    binding: usize,
    descriptor_type: VkDescriptorType,
    stages: VkFlags,
) -> VkDescriptorSetLayoutBinding {
    VkDescriptorSetLayoutBinding::new(binding, descriptor_type, stages)
}

impl VkDescriptorSetLayoutBinding {
    pub fn new(
        binding: usize,
        descriptor_type: VkDescriptorType,
        //descriptor_count: usize,
        stages: VkFlags, // VkShaderStageFlags,
    ) -> Self {
        Self {
            binding: binding as u32,
            descriptorType: descriptor_type,
            descriptorCount: 1,
            stageFlags: stages.into(),
            pImmutableSamplers: ptr::null(),
        }
    }
}

impl VkDescriptorPoolSize {
    pub fn new(descriptor_type: VkDescriptorType, count: usize) -> Self {
        Self {
            ttype: descriptor_type,
            descriptorCount: count as u32,
        }
    }
}

impl Default for VkInstanceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            pApplicationInfo: &VkApplicationInfo::default(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
        }
    }
}

impl Default for VkApplicationInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext: ptr::null(),
            pApplicationName: ptr::null(),
            applicationVersion: 0,
            pEngineName: ptr::null(),
            engineVersion: 0,
            apiVersion: 0,
        }
    }
}

impl Default for VkDebugUtilsMessengerCreateInfoEXT {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            pNext: ptr::null(),
            flags: 0,
            messageSeverity: 0.into(),
            messageType: 0.into(),
            pfnUserCallback: None,
            pUserData: ptr::null_mut(),
        }
    }
}

impl Default for VkXlibSurfaceCreateInfoKHR {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0,
            dpy: ptr::null_mut(),
            window: 0,
        }
    }
}

impl Default for VkPhysicalDeviceProperties {
    fn default() -> Self {
        Self {
            apiVersion: 0,
            driverVersion: 0,
            vendorID: 0,
            deviceID: 0,
            deviceType: VkPhysicalDeviceType::default(),
            deviceName: [0; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE],
            pipelineCacheUUID: [0; VK_UUID_SIZE],
            limits: VkPhysicalDeviceLimits::default(),
            sparseProperties: VkPhysicalDeviceSparseProperties::default(),
        }
    }
}

impl fmt::Debug for VkPhysicalDeviceProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkPhysicalDeviceProperties")
            .field("apiVersion", &vk_version_to_string(self.apiVersion))
            .field("driverVersion", &self.driverVersion)
            .field("vendorID", &self.vendorID)
            .field("deviceID", &self.deviceID)
            .field("deviceType", &self.deviceType)
            .field("deviceName", &cstr_to_string(self.deviceName.as_ptr()))
            .field("pipelineCacheUUID", &format_uuid(self.pipelineCacheUUID))
            .field("limits", &self.limits)
            .field("sparseProperties", &self.sparseProperties)
            .finish()
    }
}

impl fmt::Debug for VkPhysicalDeviceDriverProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkPhysicalDeviceDriverProperties")
            .field("sType", &self.sType)
            .field("pNext", &self.pNext)
            .field("driverID", &self.driverID)
            .field("driverName", &cstr_to_string(self.driverName.as_ptr()))
            .field("driverInfo", &cstr_to_string(self.driverInfo.as_ptr()))
            .field("conformanceVersion", &self.conformanceVersion)
            .finish()
    }
}

impl fmt::Debug for VkExtensionProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkExtensionProperties")
            .field("extensionName", &cstr_to_string(self.extensionName.as_ptr()))
            .field("specVersion", &vk_version_to_string(self.specVersion))
            .finish()
    }
}

impl fmt::Debug for VkLayerProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkLayerProperties")
            .field("layerName", &cstr_to_string(self.layerName.as_ptr()))
            .field("specVersion", &vk_version_to_string(self.specVersion))
            .field("implementationVersion", &self.implementationVersion)
            .field("description", &cstr_to_string(self.description.as_ptr()))
            .finish()
    }
}

impl fmt::Debug for VkClearValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "VkClearValue {{")?;
        let uint32 = unsafe { self.color.uint32 };
        writeln!(f, "    [{:#08x} {:#08x} {:#08x} {:#08x}]", uint32[0], uint32[1], uint32[2], uint32[3])?;
        writeln!(f, "}}")
    }
}

impl Default for VkDeviceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            queueCreateInfoCount: 0,
            pQueueCreateInfos: ptr::null(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: ptr::null(),
            enabledExtensionCount: 0,
            ppEnabledExtensionNames: ptr::null(),
            pEnabledFeatures: ptr::null(),
        }
    }
}

impl Default for VkDeviceQueueCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            queueFamilyIndex: 0,
            queueCount: 0,
            pQueuePriorities: ptr::null(),
        }
    }
}

impl Default for VkSwapchainCreateInfoKHR {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0.into(),
            surface: VkSurfaceKHR::default(),
            minImageCount: 0,
            imageFormat: VkFormat::default(),
            imageColorSpace: VkColorSpaceKHR::default(),
            imageExtent: VkExtent2D::default(),
            imageArrayLayers: 0,
            imageUsage: 0.into(),
            imageSharingMode: VkSharingMode::default(),
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            preTransform: 0.into(),
            compositeAlpha: 0.into(),
            presentMode: VkPresentModeKHR::default(),
            clipped: VK_FALSE,
            oldSwapchain: VkSwapchainKHR::default(),
        }
    }
}

impl Default for VkImageCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            imageType: VkImageType::default(),
            format: VkFormat::default(),
            extent: VkExtent3D::default(),
            mipLevels: 0,
            arrayLayers: 0,
            samples: 0.into(),
            tiling: VkImageTiling::default(),
            usage: 0.into(),
            sharingMode: VkSharingMode::default(),
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
            initialLayout: VkImageLayout::default(),
        }
    }
}

impl Default for VkImageViewCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            image: VkImage::default(),
            viewType: VkImageViewType::default(),
            format: VkFormat::default(),
            components: VkComponentMapping::default(),
            subresourceRange: VkImageSubresourceRange::default(),
        }
    }
}

impl Default for VkDescriptorSetLayoutCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            bindingCount: 0,
            pBindings: ptr::null(),
        }
    }
}

impl Default for VkDescriptorSetLayoutBinding {
    fn default() -> Self {
        Self {
            binding: 0,
            descriptorType: VkDescriptorType::default(),
            descriptorCount: 0,
            stageFlags: 0.into(),
            pImmutableSamplers: ptr::null(),
        }
    }
}

impl Default for VkRenderPassCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            attachmentCount: 0,
            pAttachments: ptr::null(),
            subpassCount: 0,
            pSubpasses: ptr::null(),
            dependencyCount: 0,
            pDependencies: ptr::null(),
        }
    }
}

impl Default for VkSubpassDescription {
    fn default() -> Self {
        Self {
            flags: 0.into(),
            pipelineBindPoint: VkPipelineBindPoint::default(),
            inputAttachmentCount: 0,
            pInputAttachments: ptr::null(),
            colorAttachmentCount: 0,
            pColorAttachments: ptr::null(),
            pResolveAttachments: ptr::null(),
            pDepthStencilAttachment: ptr::null(),
            preserveAttachmentCount: 0,
            pPreserveAttachments: ptr::null(),
        }
    }
}

impl Default for VkShaderModuleCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            codeSize: 0,
            pCode: ptr::null(),
        }
    }
}

impl Default for VkPipelineLayoutCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            setLayoutCount: 0,
            pSetLayouts: ptr::null(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: ptr::null(),
        }
    }
}

impl Default for VkGraphicsPipelineCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            stageCount: 0,
            pStages: ptr::null(),
            pVertexInputState: ptr::null(),
            pInputAssemblyState: ptr::null(),
            pTessellationState: ptr::null(),
            pViewportState: ptr::null(),
            pRasterizationState: ptr::null(),
            pMultisampleState: ptr::null(),
            pDepthStencilState: ptr::null(),
            pColorBlendState: ptr::null(),
            pDynamicState: ptr::null(),
            layout: VkPipelineLayout::default(),
            renderPass: VkRenderPass::default(),
            subpass: 0,
            basePipelineHandle: VkPipeline::default(),
            basePipelineIndex: 0,
        }
    }
}

impl Default for VkPipelineShaderStageCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            stage: 0.into(),
            module: VkShaderModule::default(),
            pName: ptr::null(),
            pSpecializationInfo: ptr::null(),
        }
    }
}

impl Default for VkPipelineVertexInputStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            vertexBindingDescriptionCount: 0,
            pVertexBindingDescriptions: ptr::null(),
            vertexAttributeDescriptionCount: 0,
            pVertexAttributeDescriptions: ptr::null(),
        }
    }
}

impl Default for VkPipelineInputAssemblyStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            topology: VkPrimitiveTopology::default(),
            primitiveRestartEnable: VK_FALSE,
        }
    }
}

impl Default for VkPipelineViewportStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            viewportCount: 0,
            pViewports: ptr::null(),
            scissorCount: 0,
            pScissors: ptr::null(),
        }
    }
}

impl Default for VkPipelineRasterizationStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            depthClampEnable: VK_FALSE,
            rasterizerDiscardEnable: VK_FALSE,
            polygonMode: VkPolygonMode::default(),
            cullMode: 0.into(),
            frontFace: VkFrontFace::default(),
            depthBiasEnable: VK_FALSE,
            depthBiasConstantFactor: 0.0,
            depthBiasClamp: 0.0,
            depthBiasSlopeFactor: 0.0,
            lineWidth: 0.0,
        }
    }
}

impl Default for VkPipelineMultisampleStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            rasterizationSamples: 0.into(),
            sampleShadingEnable: VK_FALSE,
            minSampleShading: 0.0,
            pSampleMask: ptr::null(),
            alphaToCoverageEnable: VK_FALSE,
            alphaToOneEnable: VK_FALSE,
        }
    }
}

impl Default for VkPipelineDepthStencilStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            depthTestEnable: VK_FALSE,
            depthWriteEnable: VK_FALSE,
            depthCompareOp: VkCompareOp::default(),
            depthBoundsTestEnable: VK_FALSE,
            stencilTestEnable: VK_FALSE,
            front: VkStencilOpState::default(),
            back: VkStencilOpState::default(),
            minDepthBounds: 0.0,
            maxDepthBounds: 0.0,
        }
    }
}

impl Default for VkPipelineColorBlendStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            logicOpEnable: VK_FALSE,
            logicOp: VkLogicOp::default(),
            attachmentCount: 0,
            pAttachments: ptr::null(),
            blendConstants: [0.0; 4],
        }
    }
}

impl Default for VkPipelineDynamicStateCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            dynamicStateCount: 0,
            pDynamicStates: ptr::null(),
        }
    }
}

impl Default for VkFramebufferCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            renderPass: VkRenderPass::default(),
            attachmentCount: 0,
            pAttachments: ptr::null(),
            width: 0,
            height: 0,
            layers: 0,
        }
    }
}

impl Default for VkDescriptorPoolCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            maxSets: 0,
            poolSizeCount: 0,
            pPoolSizes: ptr::null(),
        }
    }
}

impl Default for VkDescriptorSetAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: ptr::null(),
            descriptorPool: VkDescriptorPool::default(),
            descriptorSetCount: 0,
            pSetLayouts: ptr::null(),
        }
    }
}

impl Default for VkCommandPoolCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            queueFamilyIndex: 0,
        }
    }
}

impl Default for VkCommandBufferAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: ptr::null(),
            commandPool: VkCommandPool::default(),
            level: VkCommandBufferLevel::default(),
            commandBufferCount: 0,
        }
    }
}

impl Default for VkBufferCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            size: VkDeviceSize::default(),
            usage: 0.into(),
            sharingMode: VkSharingMode::default(),
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ptr::null(),
        }
    }
}

impl Default for VkMemoryAllocateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ptr::null(),
            allocationSize: VkDeviceSize::default(),
            memoryTypeIndex: 0,
        }
    }
}

impl Default for VkSemaphoreCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
        }
    }
}

impl Default for VkFenceCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
        }
    }
}

impl Default for VkSamplerCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            magFilter: VkFilter::default(),
            minFilter: VkFilter::default(),
            mipmapMode: VkSamplerMipmapMode::default(),
            addressModeU: VkSamplerAddressMode::default(),
            addressModeV: VkSamplerAddressMode::default(),
            addressModeW: VkSamplerAddressMode::default(),
            mipLodBias: 0.0,
            anisotropyEnable: VK_FALSE,
            maxAnisotropy: 0.0,
            compareEnable: VK_FALSE,
            compareOp: VkCompareOp::default(),
            minLod: 0.0,
            maxLod: 0.0,
            borderColor: VkBorderColor::default(),
            unnormalizedCoordinates: VK_FALSE,
        }
    }
}

impl Default for VkBufferMemoryBarrier {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER,
            pNext: ptr::null(),
            srcAccessMask: 0.into(),
            dstAccessMask: 0.into(),
            srcQueueFamilyIndex: 0,
            dstQueueFamilyIndex: 0,
            buffer: VkBuffer::default(),
            offset: VkDeviceSize::default(),
            size: VkDeviceSize::default(),
        }
    }
}

impl Default for VkImageMemoryBarrier {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
            pNext: ptr::null(),
            srcAccessMask: 0.into(),
            dstAccessMask: 0.into(),
            oldLayout: VkImageLayout::default(),
            newLayout: VkImageLayout::default(),
            srcQueueFamilyIndex: 0,
            dstQueueFamilyIndex: 0,
            image: VkImage::default(),
            subresourceRange: VkImageSubresourceRange::default(),
        }
    }
}

impl Default for VkCommandBufferBeginInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: ptr::null(),
            flags: 0.into(),
            pInheritanceInfo: ptr::null(),
        }
    }
}

impl Default for VkSubmitInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            pWaitDstStageMask: ptr::null(),
            commandBufferCount: 0,
            pCommandBuffers: ptr::null(),
            signalSemaphoreCount: 0,
            pSignalSemaphores: ptr::null(),
        }
    }
}

impl Default for VkPresentInfoKHR {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext: ptr::null(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ptr::null(),
            swapchainCount: 0,
            pSwapchains: ptr::null(),
            pImageIndices: ptr::null(),
            pResults: ptr::null_mut(),
        }
    }
}

impl Default for VkRenderPassBeginInfo {
    fn default() -> Self {
        VkRenderPassBeginInfo {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: ptr::null(),
            renderPass: VkRenderPass::default(),
            framebuffer: VkFramebuffer::default(),
            renderArea: VkRect2D::default(),
            clearValueCount: 0,
            pClearValues: ptr::null(),
        }
    }
}

impl Default for VkWriteDescriptorSet {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ptr::null(),
            dstSet: VkDescriptorSet::default(),
            dstBinding: 0,
            dstArrayElement: 0,
            descriptorCount: 0,
            descriptorType: VkDescriptorType::default(),
            pImageInfo: ptr::null(),
            pBufferInfo: ptr::null(),
            pTexelBufferView: ptr::null(),
        }
    }
}

pub extern "C" fn debug_callback(
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
                println!("{}", CStr::from_ptr((*p_callback_data).pMessage).to_string_lossy());
            }
        }
        VK_FALSE
    }
}
