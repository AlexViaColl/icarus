#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;

#[link(name = "vulkan")]
extern "C" {
    pub fn vkEnumerateInstanceExtensionProperties(
        pLayerName: *const i8,
        pPropertyCount: *mut u32,
        pProperties: *mut VkExtensionProperties,
    ) -> VkResult;
    pub fn vkCreateInstance(
        pCreateInfo: *const VkInstanceCreateInfo,
        pAllocator: *const VkAllocationCallbacks,
        pInstance: *mut VkInstance,
    ) -> VkResult;
}

pub const VK_MAX_EXTENSION_NAME_SIZE: usize = 256;

pub type VkFlags = u32;

#[repr(C)]
pub struct VkInstance_ {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
pub type VkInstance = *mut VkInstance_;

pub type VkInstanceCreateFlags = VkFlags;

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
pub struct VkExtensionProperties {
    extensionName: [i8; VK_MAX_EXTENSION_NAME_SIZE],
    specVersion: u32,
}

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
}
pub use VkStructureType::*;
