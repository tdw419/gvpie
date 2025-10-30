use godot::prelude::*;
use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::data_channel::data_channel_message::DataChannelMessage;

mod gpu_binary_loader;
mod syscall_translator;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct InfiniteMapOSNode {
    base: Base<Node>
}

#[godot_api]
impl INode for InfiniteMapOSNode {
    fn init(base: Base<Node>) -> Self {
        godot_print!("Hello, world!"); // Prints to the Godot console
        Self {
            base
        }
    }
}

#[godot_api]
impl InfiniteMapOSNode {
    #[func]
    fn start_webrtc(&mut self) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let api = APIBuilder::new().build();

            let config = RTCConfiguration {
                ice_servers: vec![RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                }],
                ..Default::default()
            };

            let peer_connection = api.new_peer_connection(config).await.unwrap();

            peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                godot_print!("Peer Connection State has changed: {}", s);
                Box::pin(async {})
            }));

            let data_channel = peer_connection.create_data_channel("data", None).await.unwrap();

            data_channel.on_open(Box::new(|| {
                godot_print!("Data channel is open");
                Box::pin(async {})
            }));

            data_channel.on_message(Box::new(|msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                godot_print!("Message from data channel: {}", msg_str);
                Box::pin(async {})
            }));
        });
    }

    #[func]
    fn run_cobol_demo(&mut self) {
        let (device, queue) = pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .unwrap();
            adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .unwrap()
        });

        match gpu_binary_loader::GPUBinaryLoader::load_elf_binary("cobol/hello", &device, &queue) {
            Ok(process) => {
                let syscall_translator = syscall_translator::SyscallTranslator {};
                let args = [1, 0, 0];
                let output = syscall_translator.handle_syscall(process.process_id, syscall_translator::SYS_WRITE, &args);
                godot_print!("COBOL output: {}", output);
            }
            Err(e) => {
                godot_print!("Failed to load COBOL binary: {}", e);
            }
        }
    }
}
