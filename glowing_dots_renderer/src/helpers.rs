use anyhow::{Result, anyhow};
use vulkanalia_sys as vk;

pub fn check_result(result: vk::Result) -> Result<()> {
    if result != vk::Result::SUCCESS {
        return Err(anyhow!("Got vkResult: {:?}", result));
    }

    Ok(())
}
