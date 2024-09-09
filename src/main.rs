use std::sync::Arc;

// criando uma janela e administrando a janela
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use vulkano::VulkanLibrary;

// criando uma instance do Vulkan https://vulkano.rs/02-initialization/01-initialization.html
use vulkano::instance::{Instance, InstanceCreateInfo};

// criando abstração por cima da GPU e escolhendo suas queues https://vulkano.rs/02-initialization/02-device-creation.html
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags};

// tipos de buffer https://vulkano.rs/03-buffer-creation/01-buffer-creation.html
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};

// alocador de memoria (criação de buffer) https://vulkano.rs/03-buffer-creation/01-buffer-creation.html
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

fn main() {

    // event loop da janela (winit)
    let event_loop = EventLoop::new().expect("failed to create an event_loop");

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

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
