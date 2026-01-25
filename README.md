## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
dioxus-floating = "0.1.2"
```

# dioxus-floating

A lightweight, high-performance floating positioning library for Dioxus 0.7.

## Features
- **Smart Placement**: Automatic flip and shift logic.
- **Scroll Awareness**: Works perfectly inside `ScrollableView`.
- **Portal Friendly**: Designed to work with global layer managers.
- **GPU Accelerated**: Uses `translate3d` for buttery smooth performance.

## Quick Start
```rust
use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_floating::{use_placement, FloatingOptions};

#[component]
fn App() -> Element {
    rsx! {
        ScrollableView {
            Dropdown { }
        }
    }
}
#[component]
fn Dropdown(children: Element) -> Element {
    let element_ref = use_signal(|| Option::<Rc<MountedData>>::None);
    let trigger_ref = use_signal(|| Option::<Rc<MountedData>>::None);
    
    let placement = use_placement(element_ref, trigger_ref, FloatingOptions::default());
    let element_style = use_memo(move || {
        placement.with(|pos| format!(
            "position: fixed; inset: 0px auto auto 0px; margin: 0px; transform: translate3d({}px, {}px, 0px);",
            pos.x, pos.y,
        ))
    });
    
    rsx! {
        button {
            onmounted: move |evt: MountedEvent| { trigger_ref.set(Some(evt.data.clone())) },
            "I am button!"
        }
        div {
            onmounted: move |evt: MountedEvent| { element_ref.set(Some(evt.data.clone())) },
            style: "{element_style}",
            class: if placement.is_ready { "opacity-100" } else { "opacity-0" },
            "I am floating!"
        }
    }
}
```

## Why ScrollableView?
`ScrollableView` is a required wrapper that provides a reactive context of the scrollable area. It tracks:
- **Layout changes**: Via `ResizeObserver`.
- **Scroll offsets**: Via `onscroll` event.
- **Async measurements**: Provides a `reload()` method for manual sync.

## Context Menus
Use `use_placement_on_point` to anchor elements to mouse coordinates:

```rust
let click_pos = use_signal(|| None);
let placement = use_placement_on_point(element_ref, click_pos, options);

rsx! {
    div {
        oncontextmenu: move |e| {
            e.prevent_default();
            click_pos.set(Some(e.client_coordinates()));
        },
        "Right click me"
    }
}
```

## Status
This crate is in early development (**v0.1.0**). It was built out of necessity for a complex chat application and is currently "battle-tested" there. PRs and feedback are welcome!
