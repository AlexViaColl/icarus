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

pub type VkBool32 = u32;
pub type VkFlags = u32;

#[repr(C)]
pub struct VkInstance_ {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
pub type VkInstance = *mut VkInstance_;

pub type VkInstanceCreateFlags = VkFlags;
pub type VkDebugUtilsMessageTypeFlagsEXT = VkFlags;
pub type VkDebugUtilsMessageSeverityFlagsEXT = VkFlags;
pub type VkDebugUtilsMessengerCreateFlagsEXT = VkFlags;

pub type PFN_vkVoidFunction = extern "C" fn();
pub type PFN_vkCreateDebugUtilsMessengerEXT = extern "C" fn(
    instance: VkInstance,
    pCreateInfo: *const VkDebugUtilsMessengerCreateInfoEXT,
    pAllocator: *const VkAllocationCallbacks,
    pMessenger: *mut VkDebugUtilsMessengerEXT,
) -> VkResult;
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
pub struct VkDebugUtilsMessengerCallbackDataEXT {
    _data: [u8; 0],
    //    sType: VkStructureType,
    //    pNext: *const c_void,
    //    flags: VkDebugUtilsMessengerCallbackDataFlagsEXT,
    //    pMessageIdName: *const i8,
    //    messageIdNumber: i32,
    //    pMessage: *const i8,
    //    queueLabelCount: u32,
    //    pQueueLabels: *const VkDebugUtilsLabelEXT,
    //    cmdBufLabelCount: u32,
    //    pCmdBufLabels: *const VkDebugUtilsLabelEXT,
    //    objectCount: u32,
    //    pObjects: *const VkDebugUtilsObjectNameInfoEXT,
}

// Enums
#[repr(C)]
#[derive(Debug)]
pub enum VkResult {
    VK_SUCCESS = 0,
    VK_NOT_READY = 1,
    // ...
}

#[repr(C)]
pub enum VkStructureType {
    VK_STRUCTURE_TYPE_APPLICATION_INFO = 0,
    VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO = 1,
    // ...
    VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT = 1000128004,
    // ...
}
pub use VkStructureType::*;
