use std::fmt::Debug;

use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::device::QueueFlags;
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};

fn main() {

    // cria procura se existe vulkan no computador e puxa as bibliotecas necessárias
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance = Instance::new(library, InstanceCreateInfo::default()).expect("failed to create instance");

    // escolhe um dispositivo para rodar (nesse caso o primeiro dispositivo da lista)
    // TODO: seria ideal escolher o dispositivo mais potente ou o padrão do sistema
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .next()
        .expect("no devices available");

    println!("{}", physical_device.properties().device_name);
    // cria as queues de processamento no dispositivo escolhido
    // ao mesmo tempo checa se os tipos de queue estão disponíveis
    // 
    // nesse caso so aloca para uma queue de gráficos
    let queue_family_index = physical_device
    .queue_family_properties()
    .iter()
    .enumerate()
    .position(|(_queue_family_index, queue_family_properties)| {
        queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
    })
    .expect("couldn't find a graphical queue family") as u32;

    
    // cria o wrapper de dispositivo virtual para podermos utilizar a placa de vídeo
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            // here we pass the desired queue family to use by index
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("failed to create device");

    // aqui como escolhemos apenas um tipo de queue podemos simplesmente pegar o primeiro
    // valor do vetor
    let queue = queues.next().unwrap();

}
