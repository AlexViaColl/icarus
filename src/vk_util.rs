use super::string_util::*;
use super::vk::*;

use std::fmt;
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
            .field("apiVersion", &self.apiVersion)
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

impl fmt::Debug for VkExtensionProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkExtensionProperties")
            .field("extensionName", &cstr_to_string(self.extensionName.as_ptr()))
            .field("specVersion", &self.specVersion)
            .finish()
    }
}

impl fmt::Debug for VkLayerProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkLayerProperties")
            .field("layerName", &cstr_to_string(self.layerName.as_ptr()))
            .field("specVersion", &self.specVersion)
            .field("implementationVersion", &self.implementationVersion)
            .field("description", &cstr_to_string(self.description.as_ptr()))
            .finish()
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
