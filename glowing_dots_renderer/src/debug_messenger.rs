use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use vulkanalia_sys::{self as vk, Handle};

use crate::{ALLOCATION_CALLBACKS, Instance};

struct InnerDebugUtilsMessengerEXT {
    instance: Instance,
    messenger: vk::DebugUtilsMessengerEXT,
    destroy_debug_utils_messenger_ext: vk::PFN_vkDestroyDebugUtilsMessengerEXT,
}

impl Drop for InnerDebugUtilsMessengerEXT {
    fn drop(&mut self) {
        unsafe {
            (self.destroy_debug_utils_messenger_ext)(
                self.instance.inner.instance,
                self.messenger,
                &ALLOCATION_CALLBACKS,
            );
        }
    }
}

struct DebugUtilsMessengerEXT {
    inner: Arc<InnerDebugUtilsMessengerEXT>,
}

impl DebugUtilsMessengerEXT {
    pub fn new(
        instance: &Instance,
        create_info: &vk::DebugUtilsMessengerCreateInfoEXT,
    ) -> Self {
        let destroy_debug_utils_messenger_ext = instance
            .inner
            .functions
            .vkDestroyDebugUtilsMessengerEXT
            .unwrap();

        let messenger = vk::DebugUtilsMessengerEXT::null();

        // (destroy_debug_utils_messenger_ext)(instance.inner.instance, )

        Self {
            inner: Arc::new(InnerDebugUtilsMessengerEXT {
                instance: instance.clone(),
                messenger,
                destroy_debug_utils_messenger_ext,
            }),
        }
    }
}
