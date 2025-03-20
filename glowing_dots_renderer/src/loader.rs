use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use vulkanalia_sys as vk;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct InnerLoader {
    pub vkCreateInstance: vk::PFN_vkCreateInstance,
    pub vkDestroyInstance: vk::PFN_vkDestroyInstance,
    #[allow(unused)]
    library: libloading::Library,
}

impl InnerLoader {
    unsafe fn get<T: Copy>(lib: &libloading::Library, symbol: &[u8]) -> Result<T> {
        unsafe { Ok(*lib.get::<T>(symbol)?) }
    }

    fn new() -> Result<InnerLoader> {
        let lib = unsafe { libloading::Library::new("libvulkan.so.1")? };
        
        unsafe {
            Ok(InnerLoader {
                vkCreateInstance: Self::get(&lib, b"vkCreateInstance")?,
                vkDestroyInstance: Self::get(&lib, b"vkDestroyInstance")?,
                library: lib,
            })
        }
    }
}

#[derive(Clone, Debug)]
pub struct Loader {
    inner: Arc<InnerLoader>,
}

impl Loader {
    pub fn new() -> Result<Loader> {
        Ok(Loader {
            inner: Arc::new(InnerLoader::new()?),
        })
    }
}

impl Deref for Loader {
    type Target = InnerLoader;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::check_result;
    use std::ptr::null;

    #[test]
    fn test_new() -> Result<()> {
        let loader = Loader::new()?;

        let mut instance: vk::Instance = Default::default();

        unsafe {
            check_result((loader.vkCreateInstance)(
                &vk::InstanceCreateInfo {
                    s_type: vk::StructureType::INSTANCE_CREATE_INFO,
                    application_info: &vk::ApplicationInfo {
                        s_type: vk::StructureType::APPLICATION_INFO,
                        application_name: c"Renderer Test Suite".as_ptr().cast(),
                        engine_name: c"No Engine".as_ptr().cast(),
                        api_version: vk::make_version(1, 0, 0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                null(),
                &mut instance,
            ))?;
        }

        unsafe {
            (loader.vkDestroyInstance)(instance, null());
        }

        Ok(())
    }
}
