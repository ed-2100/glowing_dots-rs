use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use libloading::Library;
use vulkanalia_sys as vk;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct LoaderFunctions {
    pub vkCreateInstance: Option<vk::PFN_vkCreateInstance>,
    pub vkDestroyInstance: Option<vk::PFN_vkDestroyInstance>,
    pub vkGetInstanceProcAddr: Option<vk::PFN_vkGetInstanceProcAddr>,
}

impl LoaderFunctions {
    unsafe fn get<T: Copy>(lib: &Library, symbol: &[u8]) -> Option<T> {
        unsafe { lib.get::<T>(symbol).ok().map(|f| *f) }
    }

    fn new(lib: &Library) -> LoaderFunctions {
        unsafe {
            LoaderFunctions {
                vkCreateInstance: Self::get(&lib, b"vkCreateInstance"),
                vkDestroyInstance: Self::get(&lib, b"vkDestroyInstance"),
                vkGetInstanceProcAddr: Self::get(&lib, b"vkGetInstanceProcAddr"),
            }
        }
    }
}

#[derive(Debug)]
pub struct InnerLoader {
    pub library: Library,
    pub functions: LoaderFunctions,
}

impl InnerLoader {
    fn new() -> Result<InnerLoader> {
        let lib = unsafe { Library::new("libvulkan.so.1")? };

        Ok(InnerLoader {
            functions: LoaderFunctions::new(&lib),
            library: lib,
        })
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

        let create_instance = loader.functions.vkCreateInstance.unwrap();
        let destroy_instance = loader.functions.vkDestroyInstance.unwrap();

        unsafe {
            check_result((create_instance)(
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
            (destroy_instance)(instance, null());
        }

        Ok(())
    }
}
