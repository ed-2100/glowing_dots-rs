use std::sync::Arc;

use anyhow::Result;
use vulkanalia_sys as vk;

use crate::{ALLOCATION_CALLBACKS, Loader, check_result};

#[allow(non_snake_case)]
#[derive(Debug)]
pub(crate) struct InnerInstance {
    pub(crate) instance: vk::Instance,
    loader: Loader,
    pub vkCreateWaylandSurfaceKHR: Option<vk::PFN_vkCreateWaylandSurfaceKHR>,
    pub vkDestroySurfaceKHR: Option<vk::PFN_vkDestroySurfaceKHR>,
}

impl InnerInstance {
    fn new(loader: &Loader, create_info: &vk::InstanceCreateInfo) -> Result<InnerInstance> {
        let mut instance = Default::default();

        unsafe {
            check_result((loader.vkCreateInstance)(
                create_info,
                &ALLOCATION_CALLBACKS,
                &mut instance,
            ))?
        };

        Ok(unsafe {
            InnerInstance {
                vkCreateWaylandSurfaceKHR: (loader.vkGetInstanceProcAddr)(
                    instance,
                    c"vkCreateWaylandSurfaceKHR".as_ptr(),
                )
                .map(|f| std::mem::transmute(f)),
                vkDestroySurfaceKHR: (loader.vkGetInstanceProcAddr)(
                    instance,
                    c"vkDestroySurfaceKHR".as_ptr(),
                )
                .map(|f| std::mem::transmute(f)),
                instance,
                loader: loader.clone(),
            }
        })
    }
}

impl Drop for InnerInstance {
    fn drop(&mut self) {
        unsafe { (self.loader.vkDestroyInstance)(self.instance, &ALLOCATION_CALLBACKS) }
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub(crate) inner: Arc<InnerInstance>,
}

impl Instance {
    pub fn new(loader: &Loader, create_info: &vk::InstanceCreateInfo) -> Result<Instance> {
        Ok(Instance {
            inner: Arc::new(InnerInstance::new(loader, create_info)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() -> Result<()> {
        let loader = Loader::new()?;

        #[allow(unused)]
        let instance = Instance::new(
            &loader,
            &vk::InstanceCreateInfo {
                application_info: &vk::ApplicationInfo {
                    application_name: c"Renderer Test Suite".as_ptr().cast(),
                    engine_name: c"No Engine".as_ptr().cast(),
                    api_version: vk::make_version(1, 0, 0),
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;

        Ok(())
    }
}
