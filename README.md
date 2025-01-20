# egui_citro3d

This repository is the home of the `egui_citro3d` project, which aims to enable mostly ordinary Rust developers who know `egui` to create graphical 3DS homebrew with relative ease.

## Structure

This repository is currently in dire need of a refactor, which should come in the next few commits.

## Getting Started

```rust
let top_viewport_id = egui::ViewportId::from_hash_of("top_viewport");
let bottom_viewport_id = egui::ViewportId::from_hash_of("bottom_viewport");
egui_citro3d::run_egui(|ctx,hid| {
    if ctx.viewport_id() == top_viewport_id {
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("Hello World on the Top Screen!");
        });
    }
    if ctx.viewport_id() == bottom_viewport_id {
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("Hello World on the Bottom Screen!");
        });
    }
})
```

## Documentation

This `README` file is all the documentation so far

## License

Following in the footsteps of the Rust 3DS team, This project is distributed under the Zlib license.
