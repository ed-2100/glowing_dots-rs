use std::sync::Arc;

use anyhow::Result;
use libloading::Library;

macro_rules! define_loader_functions {
    ($($fname:ident),*) => {
        #[allow(non_snake_case)]
        #[derive(Debug)]
        pub struct LoaderFunctions {
            $(
                pub $fname: Option<paste::paste! { ::vulkanalia_sys::[<PFN_ $fname>] }>,
            )*
        }

        impl LoaderFunctions {
            unsafe fn get<T: Copy>(lib: &Library, symbol: &[u8]) -> Option<T> {
                unsafe { lib.get::<T>(symbol).ok().map(|f| *f) }
            }

            fn new(lib: &Library) -> Self {
                unsafe {
                    Self {
                        $(
                            $fname: Self::get(&lib, stringify!($fname).as_bytes()),
                        )*
                    }
                }
            }
        }
    };
}

define_loader_functions! {
    vkCreateInstance,
    vkDestroyInstance,
    vkGetInstanceProcAddr
}

#[derive(Debug)]
pub(crate) struct InnerLoader {
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
    pub(crate) inner: Arc<InnerLoader>,
}

impl Loader {
    pub fn new() -> Result<Loader> {
        Ok(Loader {
            inner: Arc::new(InnerLoader::new()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::check_result;
    use std::ptr::null;
    use vulkanalia_sys as vk;

    #[test]
    fn test_new() -> Result<()> {
        let loader = Loader::new()?;

        let mut instance: vk::Instance = Default::default();

        let create_instance = loader.inner.functions.vkCreateInstance.unwrap();
        let destroy_instance = loader.inner.functions.vkDestroyInstance.unwrap();

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
