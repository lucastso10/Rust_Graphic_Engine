use std::sync::Arc;

// criando uma janela e administrando a janela
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use vulkano::VulkanLibrary;

// criando uma instance do Vulkan https://vulkano.rs/02-initialization/01-initialization.html
use vulkano::instance::{Instance, InstanceCreateInfo};

// criando abstração por cima da GPU e escolhendo suas queues https://vulkano.rs/02-initialization/02-device-creation.html
use vulkano::device::{physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags};
use vulkano::Version;

// tipos de buffer https://vulkano.rs/03-buffer-creation/01-buffer-creation.html
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};

// alocador de memoria (criação de buffer) https://vulkano.rs/03-buffer-creation/01-buffer-creation.html
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

use vulkano::swapchain::Surface;

fn main() {
    let event_loop = EventLoop::new();

    let required_extensions = Surface::required_extensions(&event_loop);

    // cria procura se existe vulkan no computador e puxa as bibliotecas necessárias
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance = Instance::new(
        library, 
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    ).expect("failed to create instance");

    // The objective of this example is to draw a triangle on a window. To do so, we first need to
    // create the window. We use the `WindowBuilder` from the `winit` crate to do that here.
    //
    // Before we can render to a window, we must first create a `vulkano::swapchain::Surface`
    // object from it, which represents the drawable surface of a window. For that we must wrap the
    // `winit::window::Window` in an `Arc`.
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    let mut device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    // escolhe um dispositivo para rodar (nesse caso o primeiro dispositivo da lista)
    // TODO: seria ideal escolher o dispositivo mais potente ou o padrão do sistema
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .filter(|p| {
            p.api_version() >= Version::V1_3 || p.supported_extensions().khr_dynamic_rendering
        })
        .filter(|p| {
            p.supported_extensions().contains(&device_extensions)
        })
        .next()
        .expect("no devices available");

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

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

    // escolhendo o alocador de memória que vai ser utilizado para criar e administrar os buffers
    //
    // TODO: aqui está como padrão mas deveriamos dar uma olhada e ver qual seria o melhor
    // para a nossa aplicação
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));


    // criando buffer
    //
    // nesse caso colocamos o valor 12 no buffer 
    // mas pode ser qualquer valor de qualquer tipo
    //
    // Note: In a real application you shouldn't create buffers with only 4 bytes of data.
    // Although buffers aren't expensive, you should try to group as
    // much related data as you can in the same buffer.
    // let data: i32 = 12;
    // let buffer = Buffer::from_data(
    //     memory_allocator.clone(),
    //     BufferCreateInfo {
    //         usage: BufferUsage::UNIFORM_BUFFER,
    //         ..Default::default()
    //     },
    //     AllocationCreateInfo {
    //         memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
    //             | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
    //         ..Default::default()
    //     },
    //     data,
    // )
    // .expect("failed to create buffer");

}
