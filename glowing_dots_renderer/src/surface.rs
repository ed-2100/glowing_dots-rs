use anyhow::Result;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::sync::Arc;
use vulkanalia_sys::{self as vk, Handle};

use crate::{ALLOCATION_CALLBACKS, Instance, check_result};

pub trait WindowHandle: HasDisplayHandle + HasWindowHandle {}

#[allow(non_snake_case)]
pub(crate) struct InnerSurface<'window> {
    instance: Instance,
    window: Box<dyn WindowHandle + 'window>,
    surface: vk::SurfaceKHR,
    destroy_surface_khr: vk::PFN_vkDestroySurfaceKHR,
}

impl<'window> Drop for InnerSurface<'window> {
    fn drop(&mut self) {
        unsafe {
            (self.destroy_surface_khr)(
                self.instance.inner.instance,
                self.surface,
                &ALLOCATION_CALLBACKS,
            )
        };
    }
}

pub struct Surface<'a> {
    pub(crate) inner: Arc<InnerSurface<'a>>,
}

impl<'a> Surface<'a> {
    pub fn from_window(
        instance: &Instance,
        window: Box<dyn WindowHandle + 'a>,
    ) -> Result<Surface<'a>> {
        let destroy_surface_khr = instance.inner.functions.vkDestroySurfaceKHR.unwrap();

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

                let create_wayland_surface_khr =
                    instance.inner.functions.vkCreateWaylandSurfaceKHR.unwrap();

                let mut surface = vk::SurfaceKHR::null();

                unsafe {
                    check_result((create_wayland_surface_khr)(
                        instance.inner.instance,
                        &create_info,
                        &ALLOCATION_CALLBACKS,
                        &mut surface,
                    ))?;
                }

                Ok(Surface {
                    inner: Arc::new(InnerSurface {
                        instance: instance.clone(),
                        window,
                        surface,
                        destroy_surface_khr,
                    }),
                })
            }
            _ => unimplemented!(),
        }
    }
}
