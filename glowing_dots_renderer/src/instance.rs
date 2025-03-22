use anyhow::Result;
use std::{
    collections::BTreeSet,
    ffi::{CStr, c_void},
    mem::offset_of,
    ptr::null,
    sync::Arc,
};
use vulkanalia_sys::{self as vk, ApplicationInfo};

use crate::{ALLOCATION_CALLBACKS, Loader, check_result};

macro_rules! define_instance_functions {
    ($($fname:ident),*) => {
        #[allow(non_snake_case)]
        #[derive(Debug)]
        pub struct InstanceFunctions {
            $(
                pub $fname: Option<paste::paste! { ::vulkanalia_sys::[<PFN_ $fname>] }>,
            )*
        }

        impl InstanceFunctions {
            fn new(loader: &Loader, instance: vk::Instance) -> Self {
                let get_proc_addr = loader.functions.vkGetInstanceProcAddr.unwrap();
                unsafe {
                    Self {
                        $(
                            $fname: (get_proc_addr)(
                                instance,
                                ::glowing_dots_macros::cstringify!($fname).as_ptr() as *const _
                            ).map(|p| ::std::mem::transmute(p)),
                        )*
                    }
                }
            }
        }
    };
}

// For instance functions and physical device functions.
define_instance_functions! {
    // ----- VK_KHR_surface -----
    vkDestroySurfaceKHR,
    vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
    vkGetPhysicalDeviceSurfaceFormatsKHR,
    vkGetPhysicalDeviceSurfacePresentModesKHR,
    vkGetPhysicalDeviceSurfaceSupportKHR,

    // ----- VK_KHR_wayland_surface
    vkCreateWaylandSurfaceKHR,
    vkGetPhysicalDeviceWaylandPresentationSupportKHR
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub(crate) struct InnerInstance {
    pub instance: vk::Instance,
    pub loader: Loader,
    pub functions: InstanceFunctions,
    destroy_instance: vk::PFN_vkDestroyInstance,
}

impl Drop for InnerInstance {
    fn drop(&mut self) {
        unsafe { (self.destroy_instance)(self.instance, &ALLOCATION_CALLBACKS) }
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub(crate) inner: Arc<InnerInstance>,
}

#[derive(Debug)]
pub struct InstanceBuilder<'a> {
    app_info: vk::ApplicationInfo,
    extensions: BTreeSet<&'a CStr>,
    layers: BTreeSet<&'a CStr>,
    p_next: *const c_void,
}

impl<'a> InstanceBuilder<'a> {
    pub fn new() -> Self {
        Self {
            app_info: ApplicationInfo {
                application_name: c"No Name".as_ptr(),
                application_version: vk::make_version(0, 0, 0),
                engine_name: c"No Engine".as_ptr(),
                engine_version: vk::make_version(0, 0, 0),
                api_version: vk::make_version(1, 0, 0),
                ..Default::default()
            },
            extensions: Default::default(),
            layers: Default::default(),
            p_next: null(),
        }
    }

    pub fn build(&mut self, loader: &Loader) -> Result<Instance> {
        let mut instance = Default::default();

        let layers = self.layers.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();

        let extensions = self
            .extensions
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();

        let create_instance = loader.functions.vkCreateInstance.unwrap();
        let destroy_instance = loader.functions.vkDestroyInstance.unwrap();

        unsafe {
            check_result((create_instance)(
                &vk::InstanceCreateInfo {
                    next: self.p_next,
                    application_info: &self.app_info,
                    enabled_layer_count: layers.len() as _,
                    enabled_layer_names: layers.as_ptr(),
                    enabled_extension_count: extensions.len() as _,
                    enabled_extension_names: extensions.as_ptr(),
                    ..Default::default()
                },
                &ALLOCATION_CALLBACKS,
                &mut instance,
            ))?
        };

        Ok(Instance {
            inner: Arc::new(InnerInstance {
                instance,
                functions: InstanceFunctions::new(loader, instance),
                loader: loader.clone(),
                destroy_instance,
            }),
        })
    }

    pub fn with_extension(&mut self, ext: &'a CStr) -> &mut Self {
        self.extensions.insert(ext);
        self
    }

    pub fn with_layer(&mut self, layer: &'a CStr) -> &mut Self {
        self.layers.insert(layer);
        self
    }

    pub fn with_app_name(&mut self, name: &'a CStr) -> &mut Self {
        self.app_info.application_name = name.as_ptr();
        self
    }

    pub fn with_app_version(&mut self, major: u32, minor: u32, patch: u32) -> &mut Self {
        self.app_info.application_version = vk::make_version(major, minor, patch);
        self
    }

    pub fn with_engine_name(&mut self, name: &'a CStr) -> &mut Self {
        self.app_info.engine_name = name.as_ptr();
        self
    }

    pub fn with_engine_version(&mut self, major: u32, minor: u32, patch: u32) -> &mut Self {
        self.app_info.engine_version = vk::make_version(major, minor, patch);
        self
    }

    pub fn with_api_version(&mut self, major: u32, minor: u32, patch: u32) -> &mut Self {
        self.app_info.api_version = vk::make_version(major, minor, patch);
        self
    }

    /// WARNING: Delicate code.
    pub fn with_p_next<T>(&mut self, x: &'a mut T) -> &mut Self {
        let x = x as *mut T as *const c_void;
        let x_p_next = (x as usize + offset_of!(vk::BaseInStructure, next)) as *mut *const c_void;
        unsafe { *x_p_next = self.p_next };
        self.p_next = x;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::null_mut;

    use super::*;

    unsafe extern "system" fn debug_callback(
        _severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut c_void,
    ) -> vk::Bool32 {
        unsafe {
            println!(
                "Validation layer: {}",
                CStr::from_ptr((*callback_data).message).to_str().unwrap()
            );
        }
        vk::FALSE
    }

    #[test]
    fn test_builder() -> Result<()> {
        let loader = Loader::new()?;
        let instance = InstanceBuilder::new()
            .with_extension(c"VK_KHR_surface")
            .with_extension(c"VK_KHR_wayland_surface")
            .with_extension(c"VK_EXT_debug_utils")
            .with_layer(c"VK_LAYER_KHRONOS_validation")
            .with_api_version(1, 3, 0)
            .with_app_name(c"Test App Name")
            .with_app_version(1, 1, 1)
            .with_engine_name(c"Test Engine Name")
            .with_engine_version(1, 1, 1)
            .with_p_next(&mut vk::DebugUtilsMessengerCreateInfoEXT {
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                user_callback: Some(debug_callback),
                user_data: null_mut(),
                ..Default::default()
            })
            .build(&loader)?;
        println!("{:#?}", instance);
        Ok(())
    }
}
