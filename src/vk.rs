#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;

#[link(name = "vulkan")]
extern "C" {
    pub fn vkGetInstanceProcAddr(instance: VkInstance, name: *const i8) -> PFN_vkVoidFunction;
    pub fn vkCreateInstance(
        pCreateInfo: *const VkInstanceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pInstance: *mut VkInstance,
    ) -> VkResult;
    pub fn vkDestroyInstance(instance: VkInstance, pAllocator: *const VkAllocationCallbacks);
    pub fn vkEnumerateInstanceExtensionProperties(
        pLayerName: *const i8,
        pPropertyCount: *mut u32,
        pProperties: *mut VkExtensionProperties,
    ) -> VkResult;
    pub fn vkEnumerateInstanceLayerProperties(
        pPropertyCount: *mut u32,
        pProperties: *mut VkLayerProperties,
    ) -> VkResult;
}

pub const VK_MAX_EXTENSION_NAME_SIZE: usize = 256;
pub const VK_MAX_DESCRIPTION_SIZE: usize = 256;

pub const VK_KHR_SURFACE_EXTENSION_NAME: *const i8 = b"VK_KHR_surface\0".as_ptr() as *const i8;
pub const VK_KHR_XLIB_SURFACE_EXTENSION_NAME: *const i8 = b"VK_KHR_xlib_surface\0".as_ptr() as *const i8;
pub const VK_EXT_DEBUG_UTILS_EXTENSION_NAME: *const i8 = b"VK_EXT_debug_utils\0".as_ptr() as *const i8;

pub type VkBool32 = u32;
pub type VkDeviceAddress = u64;
pub type VkDeviceSize = u64;
pub type VkFlags = u32;
pub type VkSampleMask = u32;

macro_rules! VK_DEFINE_HANDLE(
    ($name: ident) => {
        #[repr(C)]
        pub struct $name {
            _data: [u8; 0],
            _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }
    }
);

// Handles
VK_DEFINE_HANDLE!(VkInstance_);
pub type VkInstance = *mut VkInstance_;
VK_DEFINE_HANDLE!(VkPhysicalDevice_);
pub type VkPhysicalDevice = *mut VkPhysicalDevice_;
VK_DEFINE_HANDLE!(VkDevice_);
pub type VkDevice = *mut VkDevice_;
VK_DEFINE_HANDLE!(VkQueue_);
pub type VkQueue = *mut VkQueue_;
VK_DEFINE_HANDLE!(VkCommandBuffer_);
pub type VkCommandBuffer = *mut VkCommandBuffer_;

VK_DEFINE_HANDLE!(VkBuffer_);
pub type VkBuffer = *mut VkBuffer_;
VK_DEFINE_HANDLE!(VkImage_);
pub type VkImage = *mut VkImage_;
VK_DEFINE_HANDLE!(VkSemaphore_);
pub type VkSemaphore = *mut VkSemaphore_;
VK_DEFINE_HANDLE!(VkFence_);
pub type VkFence = *mut VkFence_;
VK_DEFINE_HANDLE!(VkDeviceMemory_);
pub type VkDeviceMemory = *mut VkDeviceMemory_;
VK_DEFINE_HANDLE!(VkEvent_);
pub type VkEvent = *mut VkEvent_;
VK_DEFINE_HANDLE!(VkQueryPool_);
pub type VkQueryPool = *mut VkQueryPool_;
VK_DEFINE_HANDLE!(VkBufferView_);
pub type VkBufferView = *mut VkBufferView_;
VK_DEFINE_HANDLE!(VkImageView_);
pub type VkImageView = *mut VkImageView_;
VK_DEFINE_HANDLE!(VkShaderModule_);
pub type VkShaderModule = *mut VkShaderModule_;
VK_DEFINE_HANDLE!(VkPipelineCache_);
pub type VkPipelineCache = *mut VkPipelineCache_;
VK_DEFINE_HANDLE!(VkPipelineLayout_);
pub type VkPipelineLayout = *mut VkPipelineLayout_;
VK_DEFINE_HANDLE!(VkPipeline_);
pub type VkPipeline = *mut VkPipeline_;
VK_DEFINE_HANDLE!(VkRenderPass_);
pub type VkRenderPass = *mut VkRenderPass_;
VK_DEFINE_HANDLE!(VkDescriptorSet_);
pub type VkDescriptorSet = *mut VkDescriptorSet_;
VK_DEFINE_HANDLE!(VkDescriptorPool_);
pub type VkDescriptorPool = *mut VkDescriptorPool_;
VK_DEFINE_HANDLE!(VkFramebuffer_);
pub type VkFramebuffer = *mut VkFramebuffer_;
VK_DEFINE_HANDLE!(VkCommandPool_);
pub type VkCommandPool = *mut VkCommandPool_;

pub type VkInstanceCreateFlags = VkFlags;
pub type VkDebugUtilsMessageTypeFlagsEXT = VkFlags;
pub type VkDebugUtilsMessageSeverityFlagsEXT = VkFlags;
pub type VkDebugUtilsMessengerCreateFlagsEXT = VkFlags;
pub type VkDebugUtilsMessengerCallbackDataFlagsEXT = VkFlags;

pub type PFN_vkVoidFunction = extern "C" fn();
pub type PFN_vkCreateDebugUtilsMessengerEXT = extern "C" fn(
    instance: VkInstance,
    pCreateInfo: *const VkDebugUtilsMessengerCreateInfoEXT,
    pAllocator: *const VkAllocationCallbacks,
    pMessenger: *mut VkDebugUtilsMessengerEXT,
) -> VkResult;
pub type PFN_vkDestroyDebugUtilsMessengerEXT =
    extern "C" fn(instance: VkInstance, messenger: VkDebugUtilsMessengerEXT, pAllocator: *const VkAllocationCallbacks);
pub type PFN_vkDebugUtilsMessengerCallbackEXT = extern "C" fn(
    messageSeverity: VkDebugUtilsMessageSeverityFlagsEXT, // VkDebugUtilsMessageSeverityFlagBitsEXT,
    messageTypes: VkDebugUtilsMessageTypeFlagsEXT,
    pCallbackData: *const VkDebugUtilsMessengerCallbackDataEXT,
    pUserData: *mut c_void,
) -> VkBool32;

#[repr(C)]
pub struct VkInstanceCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkInstanceCreateFlags,
    pub pApplicationInfo: *const VkApplicationInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const *const i8,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const *const i8,
}

#[repr(C)]
pub struct VkApplicationInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub pApplicationName: *const i8,
    pub applicationVersion: u32,
    pub pEngineName: *const i8,
    pub engineVersion: u32,
    pub apiVersion: u32,
}

#[repr(C)]
pub struct VkAllocationCallbacks {
    _data: [u8; 0],
    // TODO
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct VkExtensionProperties {
    pub extensionName: [i8; VK_MAX_EXTENSION_NAME_SIZE],
    pub specVersion: u32,
}

impl Default for VkExtensionProperties {
    fn default() -> Self {
        Self {
            extensionName: [0; VK_MAX_EXTENSION_NAME_SIZE],
            specVersion: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct VkLayerProperties {
    pub layerName: [i8; VK_MAX_EXTENSION_NAME_SIZE],
    pub specVersion: u32,
    pub implementationVersion: u32,
    pub description: [i8; VK_MAX_DESCRIPTION_SIZE],
}

impl Default for VkLayerProperties {
    fn default() -> Self {
        Self {
            layerName: [0; VK_MAX_EXTENSION_NAME_SIZE],
            specVersion: 0,
            implementationVersion: 0,
            description: [0; VK_MAX_DESCRIPTION_SIZE],
        }
    }
}

#[repr(C)]
pub struct VkDebugUtilsMessengerEXT_ {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
pub type VkDebugUtilsMessengerEXT = *mut VkDebugUtilsMessengerEXT_;

#[repr(C)]
pub struct VkDebugUtilsMessengerCreateInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkDebugUtilsMessengerCreateFlagsEXT,
    pub messageSeverity: VkDebugUtilsMessageSeverityFlagsEXT,
    pub messageType: VkDebugUtilsMessageTypeFlagsEXT,
    pub pfnUserCallback: PFN_vkDebugUtilsMessengerCallbackEXT,
    pub pUserData: *mut c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct VkDebugUtilsMessengerCallbackDataEXT {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkDebugUtilsMessengerCallbackDataFlagsEXT,
    pub pMessageIdName: *const i8,
    pub messageIdNumber: i32,
    pub pMessage: *const i8,
    pub queueLabelCount: u32,
    pub pQueueLabels: *const VkDebugUtilsLabelEXT,
    pub cmdBufLabelCount: u32,
    pub pCmdBufLabels: *const VkDebugUtilsLabelEXT,
    pub objectCount: u32,
    pub pObjects: *const VkDebugUtilsObjectNameInfoEXT,
}

#[repr(C)]
pub struct VkDebugUtilsLabelEXT {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub pLabelName: *const i8,
    pub color: [f32; 4],
}

#[repr(C)]
pub struct VkDebugUtilsObjectNameInfoEXT {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub objectType: VkObjectType,
    pub objectHandle: u64,
    pub pObjectName: *const i8,
}

// Enums
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum VkResult {
    VK_SUCCESS = 0,
    VK_NOT_READY = 1,
    // TODO
}
pub use VkResult::*;

#[repr(C)]
#[derive(Debug)]
pub enum VkStructureType {
    VK_STRUCTURE_TYPE_APPLICATION_INFO = 0,
    VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO = 1,
    // ...
    VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT = 1000128004,
    // TODO
}
pub use VkStructureType::*;

#[repr(C)]
pub enum VkObjectType {
    VK_OBJECT_TYPE_UNKNOWN = 0,
    VK_OBJECT_TYPE_INSTANCE = 1,
    VK_OBJECT_TYPE_PHYSICAL_DEVICE = 2,
    VK_OBJECT_TYPE_DEVICE = 3,
    VK_OBJECT_TYPE_QUEUE = 4,
    VK_OBJECT_TYPE_SEMAPHORE = 5,
    VK_OBJECT_TYPE_COMMAND_BUFFER = 6,
    VK_OBJECT_TYPE_FENCE = 7,
    VK_OBJECT_TYPE_DEVICE_MEMORY = 8,
    VK_OBJECT_TYPE_BUFFER = 9,
    VK_OBJECT_TYPE_IMAGE = 10,
    VK_OBJECT_TYPE_EVENT = 11,
    VK_OBJECT_TYPE_QUERY_POOL = 12,
    VK_OBJECT_TYPE_BUFFER_VIEW = 13,
    VK_OBJECT_TYPE_IMAGE_VIEW = 14,
    VK_OBJECT_TYPE_SHADER_MODULE = 15,
    VK_OBJECT_TYPE_PIPELINE_CACHE = 16,
    VK_OBJECT_TYPE_PIPELINE_LAYOUT = 17,
    VK_OBJECT_TYPE_RENDER_PASS = 18,
    VK_OBJECT_TYPE_PIPELINE = 19,
    VK_OBJECT_TYPE_DESCRIPTOR_SET_LAYOUT = 20,
    VK_OBJECT_TYPE_SAMPLER = 21,
    VK_OBJECT_TYPE_DESCRIPTOR_POOL = 22,
    VK_OBJECT_TYPE_DESCRIPTOR_SET = 23,
    VK_OBJECT_TYPE_FRAMEBUFFER = 24,
    VK_OBJECT_TYPE_COMMAND_POOL = 25,
    // TODO
}
pub use VkObjectType::*;

pub const VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT: VkFlags = 0x00000001;
pub const VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT: VkFlags = 0x00000010;
pub const VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT: VkFlags = 0x00000100;
pub const VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT: VkFlags = 0x00001000;
pub const VK_DEBUG_UTILS_MESSAGE_SEVERITY_FLAG_BITS_MAX_ENUM_EXT: VkFlags = 0x7FFFFFF;

pub const VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT: VkFlags = 0x00000001;
pub const VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT: VkFlags = 0x00000002;
pub const VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT: VkFlags = 0x00000004;
pub const VK_DEBUG_UTILS_MESSAGE_TYPE_FLAG_BITS_MAX_ENUM_EXT: VkFlags = 0x7FFFFFF;
