# yam

`yam` is a `game engine` which uses ECS(Entity-Component-System) architecture dedicated to running millions of units.

## The State

`yam` is in the early stages of development, feature-less, documentation-sparse and change-quick. You can try to play with it, run the examples to know the basic information of `yam`.

**Any suggestions or questions are welcome**.

## Libraries Used

* [legion](https://github.com/amethyst/legion): easy to use, feature-rich and high-performance ECS `library.
* [nalgebra](https://github.com/dimforge/nalgebra): amazing, powerful and high-performance algebra library.
* [wgpu-rs](https://github.com/gfx-rs/wgpu-rs): low-level, cross-platform and modern graphics library.
* [winit](https://github.com/rust-windowing/winit): cross-platform window creation and management in Rust.

## The Example

1. **app**: Show the basic architecture of `yam`.

    ```bash
    cargo run --example app
    ```

2. **input**: Show how to iterat with the mouse and keyboard.

    ```bash
    cargo run --example input
    ```

3. **time**: Show how to get the accurate time consuming of the last frame.

    ```bash
    cargo run --example time
    ```

4. **window**: Show how to get or modify the properties of window.

    ```bash
    cargo run --example window
    ```

5. **render2d**: Show how to render a sprite to screen.

    ```bash
    cargo run --example render2d
    ```

6. **render2d_millions**: Show how to render millions of sprites to screen in 60fps.

    ```bash
    # Use `--release` flag to prevent performance decline.
    cargo run --example render2d_millions --release
    ```
