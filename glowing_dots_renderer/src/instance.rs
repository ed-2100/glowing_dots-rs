use std::{ops::Deref, sync::Arc};

use crate::{allocator::ALLOCATION_CALLBACKS, helpers::check_result, loader::Loader};
use anyhow::Result;
use vulkanalia_sys as vk;

pub struct InnerInstance {
    instance: vk::Instance,
    loader: Loader,
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

        Ok(InnerInstance {
            instance,
            loader: loader.clone(),
        })
    }
}

impl Drop for InnerInstance {
    fn drop(&mut self) {
        unsafe { (self.loader.vkDestroyInstance)(self.instance, &ALLOCATION_CALLBACKS) }
    }
}

pub struct Instance {
    inner: Arc<InnerInstance>,
}

impl Instance {
    pub fn new(loader: &Loader, create_info: &vk::InstanceCreateInfo) -> Result<Instance> {
        Ok(Instance {
            inner: Arc::new(InnerInstance::new(loader, create_info)?),
        })
    }
}

impl Deref for Instance {
    type Target = InnerInstance;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
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
