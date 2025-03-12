use std::sync::Arc;

use vulkanalia::vk;

// unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<Instance> {
//     let application_info = vk::ApplicationInfo::builder()
//         .application_name(b"Vulkan Tutorial\0")
//         .application_version(vk::make_version(1, 0, 0))
//         .engine_name(b"No Engine\0")
//         .engine_version(vk::make_version(1, 0, 0))
//         .api_version(vk::make_version(1, 0, 0));
// }

struct Entry {
    
}

struct Instance(Arc<InnerInstance>);

struct InnerInstance {}
