mod downloader;
mod dos_lib;
// 引入winit库
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn hello_windows() {
    // 创建事件循环
    let event_loop = EventLoop::new();
    
    // 创建窗口
    let window = WindowBuilder::new()
        .with_title("Hello World!")  // 窗口标题
        .build(&event_loop)
        .unwrap();

    // 事件循环，控制窗口的生命周期
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,  // 处理关闭窗口事件
                ..
            } => {
                *control_flow = ControlFlow::Exit;  // 退出事件循环，关闭程序
            }
            _ => {}
        }
    });
}

fn main() {
    hello_windows()
    }