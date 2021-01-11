# ROADMAP

## window模块使用示例

```rust
fn main() {
    let window = window::Window::default();

    // 获取/设置window的大小, 位置, 分辨率
    let size = window.get_size();
    let position = window.get_position();
    let resolution = window.get_resolution();
    window.set_size(...);
    window.set_position(...);
    window.set_resolution(...);

    // 刷新window, window会返回玩家对窗口的交互
    let input = window.refresh();

    // 关闭窗口
    window.close();
}
```

## AppBuilder模式

```rust
fn main() {
    AppBuilder::build()
        .set_render_framerate(...)
        .add_startup_system(...)
        .add_update_system(...)
        .run();
}
```

## 处理输入事件

```rust
#[system]
fn process_input(#[resource] input: &Input) {
    // process mouse event
    if input.mouse.just_pressed(KeyCode::LeftButton) {
        // do something
    }

    if input.mouse.just_released(KeyCode::LetButton) {
        // do something
    }

    if input.mouse.pressed(KeyCode::LeftButton) {
        // do something
    }

    if input.mouse.released(KeyCode::LeftButton) {
        // do something
    }

    if input.keyboard.just_pressed(KeyCode::F) {
        // do something
    }

    if input.keyboard.just_released(KeyCode::F) {
        // do something
    }

    if input.keyboard.pressed(KeyCode::F) {
        // do something
    }

    if input.keyboard.released(KeyCode::F) {
        // do something
    }
}
```

## 处理特殊窗口事件

需要注意的是, winit的事件处理循环会在一次循环中处理多种事件, 所以需要迭代window_events

```rust
#[system]
fn process_window_events(#[resource] window_events: &WindowEvents) {
    for window_event in window_events.iter() {
        match window_event {
            WindowEvent::Exit => {},
            _ => {},
        }
    }
}
```

## 设置起始system

```rust
fn main() {
    App::build()
        .add_start_system(startup_system())
        .finish()
        .run();
}
```

## 设置循环system

```rust
// 设计的目标之一: 不希望有空的(无可行system)Update Layer
fn main() {
    App::build()
        // 为UpdateStage::{PRE_UPDATE, UPDATE, POST_UPDATE}添加Layer,
        // 每个Stage最多有8层Layer, 每层Layer有单独的更新频率
        // 添加的update_layer默认更新频率是60
        .add_layer_to_update(UpdateStage::UPDATE, "layer_name", 60)
        .add_update_system(update_system(), UpdateStage::UPDATE, "layer_name")
        .finish()
        .run();
}

// 运行时修改Update的更新频率
#[system]
fn change_update_layer_frequency(#[resource] app_settings: &AppSettings) {
    app_settings.set_update_layer_frequency(UpdateStage::UPDATE, "layer_name", 90);
}
```

## 设置渲染帧率

```rust
// RenderFramerate {LOW, NORMAL, HIGH, CUSTOM(u32)};
// LOW = 30fps, NORMAL = 60fps, HIGH = 144fps, CUSTOM(u32) = u32fps
fn main() {
    App::build()
        .set_render_framerate(RenderFramerate::NORMAL)
        .finish()
        .run();
}

// 运行时修改渲染帧率
#[system]
fn change_render_framerate(#[resource] app_settings: &AppSettings) {
    app_settings.set_render_framerate(RenderFrameRate::HIGH);
}
```

## 窗口模块

```rust
fn run() {
    ...
    
    let mut window = Window::new();
    let input = window.run_returned();
    ...
    // TODO: Update Window To App Settings
    ...
}
```