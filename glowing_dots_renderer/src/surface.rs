use anyhow::Result;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::sync::Arc;
use vulkanalia_sys as vk;

use crate::{ALLOCATION_CALLBACKS, Instance};

pub trait WindowHandle: HasDisplayHandle + HasWindowHandle {}

struct InnerSurface<'window> {
    instance: Instance,
    window: Box<dyn WindowHandle + 'window>,
    surface: vk::SurfaceKHR,
}

impl<'window> Drop for InnerSurface<'window> {
    fn drop(&mut self) {
        unsafe {
            (self.instance.inner.vkDestroySurfaceKHR.unwrap())(
                self.instance.inner.instance,
                self.surface,
                &ALLOCATION_CALLBACKS,
            )
        };
    }
}

pub struct Surface<'a> {
    inner: Arc<InnerSurface<'a>>,
}

impl<'a> Surface<'a> {
    pub fn from_window(
        instance: &Instance,
        window: Box<dyn WindowHandle + 'a>,
    ) -> Result<Surface<'a>> {
        instance.inner.vkDestroySurfaceKHR.unwrap();

        match (
            window.display_handle().map(|handle| handle.as_raw()),
            window.window_handle().map(|handle| handle.as_raw()),
        ) {
            (
                Ok(RawDisplayHandle::Wayland(display)),
                Ok(RawWindowHandle::Wayland(window_handle)),
            ) => {
                let create_info = vk::WaylandSurfaceCreateInfoKHR {
                    display: display.display.as_ptr(),
                    surface: window_handle.surface.as_ptr(),
                    ..Default::default()
                };
                let mut surface = Default::default();
                let result = unsafe {
                    (instance.inner.vkCreateWaylandSurfaceKHR.unwrap())(
                        instance.inner.instance,
                        &create_info,
                        &ALLOCATION_CALLBACKS,
                        &mut surface,
                    )
                };
                if result != vk::Result::SUCCESS {
                    return Err(result.into());
                }
                Ok(Surface {
                    inner: Arc::new(InnerSurface {
                        instance: instance.clone(),
                        window,
                        surface,
                    }),
                })
            }
            _ => unimplemented!(),
        }
    }
}
