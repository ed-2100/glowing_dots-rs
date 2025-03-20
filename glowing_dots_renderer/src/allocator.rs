use std::{
    alloc::{Layout, alloc, dealloc, realloc},
    os::raw::c_void,
    ptr::null_mut,
};

use vulkanalia_sys as vk;

const HEADER_ALIGNMENT: usize = align_of::<Layout>();
const PADDED_HEADER_SIZE: usize =
    (std::mem::size_of::<Layout>() + HEADER_ALIGNMENT - 1) / HEADER_ALIGNMENT * HEADER_ALIGNMENT;

unsafe extern "system" fn allocation_function(
    _user_data: *mut c_void,
    size: usize,
    alignment: usize,
    _allocation_scope: vk::SystemAllocationScope,
) -> *mut c_void {
    if size == 0 {
        return null_mut();
    }

    if alignment.count_ones() != 1 {
        return null_mut();
    }

    if size > (isize::MAX as usize + 1) - alignment {
        return null_mut();
    }

    let aligned_offset = (PADDED_HEADER_SIZE + alignment - 1) / alignment * alignment;
    let total_size = aligned_offset + size;

    let layout =
        unsafe { Layout::from_size_align_unchecked(total_size, alignment.max(HEADER_ALIGNMENT)) };

    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    let user_ptr = (ptr as usize + aligned_offset) as *mut u8;
    let layout_ptr = (user_ptr as usize - PADDED_HEADER_SIZE) as *mut Layout;

    unsafe { *layout_ptr = layout };

    user_ptr as *mut c_void
}

unsafe extern "system" fn free_function(_user_data: *mut c_void, memory: *mut c_void) {
    if memory.is_null() {
        return;
    }

    let layout = unsafe { *((memory as usize - PADDED_HEADER_SIZE) as *mut Layout) };

    let alignment = layout.align();

    let aligned_offset = (PADDED_HEADER_SIZE + alignment - 1) / alignment * alignment;

    let ptr = (memory as usize - aligned_offset) as *mut u8;

    unsafe { dealloc(ptr, layout) };
}

unsafe extern "system" fn reallocation_function(
    _user_data: *mut c_void,
    memory: *mut c_void,
    size: usize,
    requested_alignment: usize,
    allocation_scope: vk::SystemAllocationScope,
) -> *mut c_void {
    if memory.is_null() {
        return unsafe {
            allocation_function(_user_data, size, requested_alignment, allocation_scope)
        };
    }

    if size == 0 {
        unsafe { free_function(_user_data, memory) };
        return memory;
    }

    let original_layout = unsafe { *((memory as usize - PADDED_HEADER_SIZE) as *mut Layout) };

    let alignment = original_layout.align();

    if requested_alignment != alignment {
        return null_mut();
    }

    let aligned_offset = (PADDED_HEADER_SIZE + alignment - 1) / alignment * alignment;

    let mut ptr = (memory as usize - aligned_offset) as *mut u8;

    let new_size = aligned_offset + size;

    ptr = unsafe { realloc(ptr, original_layout, new_size) };
    if ptr.is_null() {
        return null_mut();
    }

    unsafe {
        let user_ptr = (ptr as usize + aligned_offset) as *mut u8;
        let layout_ptr = (user_ptr as usize - PADDED_HEADER_SIZE) as *mut Layout;
        (*layout_ptr) = Layout::from_size_align_unchecked(new_size, alignment)
    }

    (ptr as usize + aligned_offset) as *mut c_void
}

const fn create_callbacks() -> vk::AllocationCallbacks {
    vk::AllocationCallbacks {
        user_data: std::ptr::null_mut(),
        allocation: Some(allocation_function),
        reallocation: Some(reallocation_function),
        free: Some(free_function),
        internal_allocation: None,
        internal_free: None,
    }
}

pub struct SyncAllocationCallbacks(pub vk::AllocationCallbacks);

unsafe impl Sync for SyncAllocationCallbacks {}
unsafe impl Send for SyncAllocationCallbacks {}

static ALLOCATION_CALLBACKS_INNER: SyncAllocationCallbacks =
    SyncAllocationCallbacks(create_callbacks());

pub const ALLOCATION_CALLBACKS: vk::AllocationCallbacks = ALLOCATION_CALLBACKS_INNER.0;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, anyhow};

    #[test]
    fn test_alloc() -> Result<()> {
        let sizes = [1, 2, 123, 232, 145, 128, 512];

        for size in sizes {
            for i in (1..=16).map(|i| 1 << i) {
                let mem = unsafe {
                    allocation_function(null_mut(), size, i, vk::SystemAllocationScope::OBJECT)
                };
                if mem.is_null() {
                    return Err(anyhow!(
                        "Failed to allocate w/ align = {} and size = {}",
                        i,
                        size
                    ));
                }

                unsafe { free_function(null_mut(), mem) };
            }
        }

        Ok(())
    }

    #[test]
    fn test_alloc_size_0() -> Result<()> {
        unsafe {
            let mem = allocation_function(null_mut(), 0, 8, vk::SystemAllocationScope::OBJECT);
            if !mem.is_null() {
                free_function(null_mut(), mem);
                return Err(anyhow!("Expected allocation to fail, because size is 0."));
            }
        }
        Ok(())
    }

    #[test]
    fn test_alloc_align_non_pow_two() -> Result<()> {
        unsafe {
            let mem = allocation_function(null_mut(), 500, 7, vk::SystemAllocationScope::OBJECT);
            if !mem.is_null() {
                free_function(null_mut(), mem);
                return Err(anyhow!(
                    "Expected allocation to fail, because alignment is not a power of 2."
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn test_alloc_align_lt_8() -> Result<()> {
        unsafe {
            let mem = allocation_function(null_mut(), 500, 4, vk::SystemAllocationScope::OBJECT);
            if mem.is_null() {
                return Err(anyhow!("Expected allocation to succeed."));
            }
            free_function(null_mut(), mem);
        }
        Ok(())
    }

    #[test]
    fn test_realloc_ptr_null() -> Result<()> {
        unsafe {
            let mem = reallocation_function(
                null_mut(),
                null_mut(),
                500,
                8,
                vk::SystemAllocationScope::OBJECT,
            );
            if mem.is_null() {
                return Err(anyhow!("Expected to allocate new memory when ptr is null."));
            }
            free_function(null_mut(), mem);
        }
        Ok(())
    }

    #[test]
    fn test_realloc_size_0() -> Result<()> {
        unsafe {
            let mem = allocation_function(null_mut(), 500, 8, vk::SystemAllocationScope::OBJECT);
            let new_mem =
                reallocation_function(null_mut(), mem, 0, 8, vk::SystemAllocationScope::OBJECT);
            if mem != new_mem {
                free_function(null_mut(), mem);
                return Err(anyhow!(
                    "Expected reallocate to return null and free the memory when size is 0."
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn test_realloc_align_different() -> Result<()> {
        unsafe {
            let mem = allocation_function(null_mut(), 500, 8, vk::SystemAllocationScope::OBJECT);
            let new_mem =
                reallocation_function(null_mut(), mem, 300, 4, vk::SystemAllocationScope::OBJECT);
            if !new_mem.is_null() {
                return Err(anyhow!(
                    "Expected reallocate to return null when the alignment is different."
                ));
            }
        }
        Ok(())
    }
}
