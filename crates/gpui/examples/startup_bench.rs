#![cfg_attr(target_family = "wasm", no_main)]

// Measures GPUI startup cost from `main()` entry to the first `App` callback,
// and from `main()` entry to the first rendered frame. Used to benchmark the
// effect of loading the font system on a background thread. See
// `script/bench-startup` for the runner.
//
// Environment variables honored by gpui_linux when this is run:
//   GPUI_STARTUP_BLOCKING_FONTS=1  — load fonts inline (pre-optimization baseline)
//   GPUI_STARTUP_PROFILE=1         — emit timing breakdown to stderr

use std::time::Instant;

use gpui::{
    App, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;

struct BenchView {
    title: SharedString,
}

impl Render for BenchView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Simulate a realistic first frame: title bar + menu row + body with
        // multi-size text + shape grid. This exercises glyph shaping,
        // rasterization, multiple font sizes, borders, and shadows so
        // render_ms reflects a real application's cold first-frame cost.
        let menu_item = |label: &'static str| {
            div()
                .px_2()
                .py_1()
                .text_sm()
                .text_color(rgb(0xe0e0e0))
                .child(label)
        };

        let swatch = |color| {
            div()
                .size_8()
                .bg(color)
                .border_1()
                .border_dashed()
                .rounded_md()
                .border_color(gpui::white())
        };

        div()
            .flex()
            .flex_col()
            .bg(rgb(0x202020))
            .size(px(500.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x303030))
                    .border_b_1()
                    .border_color(rgb(0x505050))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xffffff))
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(div().size_3().rounded_full().bg(rgb(0xff5f56)))
                            .child(div().size_3().rounded_full().bg(rgb(0xffbd2e)))
                            .child(div().size_3().rounded_full().bg(rgb(0x27c93f))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .bg(rgb(0x282828))
                    .child(menu_item("File"))
                    .child(menu_item("Edit"))
                    .child(menu_item("Selection"))
                    .child(menu_item("View"))
                    .child(menu_item("Help")),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .child(
                        div()
                            .text_xl()
                            .text_color(rgb(0xffffff))
                            .child("Startup benchmark"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xc0c0c0))
                            .child(
                                "The quick brown fox jumps over the lazy dog. 0123456789 \
                                 AaBbCcDdEeFfGgHh — measuring first-frame cost for GPUI.",
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(swatch(gpui::red()))
                            .child(swatch(gpui::green()))
                            .child(swatch(gpui::blue()))
                            .child(swatch(gpui::yellow()))
                            .child(swatch(gpui::white())),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(0x3a6cf0))
                                    .text_sm()
                                    .text_color(gpui::white())
                                    .child("Primary"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(0x606060))
                                    .text_sm()
                                    .text_color(rgb(0xe0e0e0))
                                    .child("Secondary"),
                            ),
                    ),
            )
    }
}

fn run_example(t0: Instant) {
    application().run(move |cx: &mut App| {
        let init_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("STARTUP_INIT_MS={:.3}", init_ms);

        let bounds = Bounds::centered(None, size(px(400.0), px(400.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                window.on_next_frame(move |_window, cx| {
                    let render_ms = t0.elapsed().as_secs_f64() * 1000.0;
                    println!("STARTUP_RENDER_MS={:.3}", render_ms);
                    cx.quit();
                });
                cx.new(|_| BenchView {
                    title: "Startup Bench".into(),
                })
            },
        )
        .unwrap();
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    let t0 = Instant::now();
    run_example(t0);
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    let t0 = Instant::now();
    gpui_platform::web_init();
    run_example(t0);
}
