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