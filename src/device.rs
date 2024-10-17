use std::sync::Arc;

use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
    },
    instance::Instance,
    memory::allocator::StandardMemoryAllocator,
    swapchain::Surface,
};

pub struct GPU {
    pub physical_device: Arc<PhysicalDevice>,
    pub logical_device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
}

impl GPU {
    pub fn new(
        device_extensions: DeviceExtensions,
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
    ) -> Self {
        // escolhe a GPU que vai utilizar
        //
        // physical device
        let (pd, queue_family_index) =
            Self::select_physical_device(&instance, &surface, &device_extensions);

        // cria o logical device e extrai a queue
        //
        // logical device
        // queue creation
        let (device, mut queues) = Device::new(
            pd.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create device");

        // como só temos uma queue pega a primeira do vetor
        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        Self {
            physical_device: pd,
            logical_device: device,
            graphics_queue: queue,
            memory_allocator,
        }
    }

    // normalmente quando vc clona essa struct vc quer o logical_device
    pub fn clone(&self) -> Arc<Device> {
        self.logical_device.clone()
    }

    fn select_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> (Arc<PhysicalDevice>, u32) {
        instance
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
            // garante que o dispositivo tem as extensões necessárias para
            // nossa aplicação
            .filter(|p| p.supported_extensions().contains(device_extensions))
            // garante que o dispositivo tenha queue de gráficos
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            // rankeia as melhores opções de GPU
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .expect("no device available")
    }
}
