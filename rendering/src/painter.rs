pub struct Painter {
    device: wgpu::Device,
    queue: wgpu::Queue
}

impl Painter {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: None
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &Default::default(),
            None
        ).await.unwrap();

        Self {
            device,
            queue
        }
    }
}
