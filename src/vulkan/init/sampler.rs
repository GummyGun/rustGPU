use crate::AAError;
use crate::logger;
use crate::macros;


use super::VkDestructor;
use super::VkDestructorArguments;
use super::Device;

use ash::vk;

pub struct Sampler {
    sampler: vk::Sampler,
}

macros::impl_underlying!(Sampler, vk::Sampler, sampler);


impl Sampler {
    pub fn create(device:&mut Device, filter:vk::Filter) -> Result<Self, AAError> {
        logger::create!("sampler");
        let sampler_ci = vk::SamplerCreateInfo::builder()
            .mag_filter(filter)
            .min_filter(filter);
        let holder = unsafe{device.create_sampler(&sampler_ci, None)}?;
        Ok(Self{
            sampler: holder,
        })
    }
}


impl VkDestructor for Sampler {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("sampler");
        let device = args.unwrap_dev();
        unsafe{device.destroy_sampler(self.underlying(), None)}
    }
}
