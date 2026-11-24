use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_floating::{
    FloatingOptions, Middleware, OffsetOptions, Placement, ScrollState, ScrollableView,
    use_placement, use_scroll_context, use_scroll_state,
};

fn main() {
    launch(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    None,
    Flip,
    Shift,
    Both,
}

impl Mode {
    const ALL: [Self; 4] = [Self::None, Self::Flip, Self::Shift, Self::Both];

    fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Flip => "Flip",
            Self::Shift => "Shift",
            Self::Both => "Flip + Shift",
        }
    }

    fn middleware(self) -> Vec<Middleware> {
        match self {
            Self::None => vec![],
            Self::Flip => vec![Middleware::Flip],
            Self::Shift => vec![Middleware::Shift],
            Self::Both => vec![Middleware::Flip, Middleware::Shift],
        }
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Middleware playground · dioxus-floating" }
        style { {include_str!("middleware.css")} }
        ScrollableView { class: "lab-page", Lab {} }
    }
}

#[component]
fn Lab() -> Element {
    let mut mode = use_signal(|| Mode::Both);
    let page_scroll = use_scroll_state();
    let active = mode();

    rsx! {
        main { class: "lab",
            header { class: "lab-header",
                a { class: "examples-link", href: "../", "dioxus-floating / Examples" }
                h1 { "Middleware playground" }
                p { "Change the middleware, then scroll each box to move its anchor past an edge." }
            }
            div { class: "controls", role: "group", aria_label: "Middleware",
                span { class: "controls-label", "Middleware" }
                div { class: "modes",
                    for option in Mode::ALL {
                        button {
                            r#type: "button",
                            aria_pressed: active == option,
                            onclick: move |_| mode.set(option),
                            {option.label()}
                        }
                    }
                }
            }
            div { class: "boards",
                ScrollDemo {
                    direction: "vertical",
                    title: "Vertical scroll",
                    placement_label: "BottomStart",
                    placement: Placement::BottomStart,
                    mode: active,
                    page_scroll,
                }
                ScrollDemo {
                    direction: "horizontal",
                    title: "Horizontal scroll",
                    placement_label: "RightStart",
                    placement: Placement::RightStart,
                    mode: active,
                    page_scroll,
                }
            }
            dl { class: "legend",
                div { dt { "Flip" } dd { "Moves the preview to the opposite side of its anchor." } }
                div { dt { "Shift" } dd { "Slides the preview along the edge of its scroll box." } }
            }
        }
    }
}

#[component]
fn ScrollDemo(
    direction: &'static str,
    title: &'static str,
    placement_label: &'static str,
    placement: Placement,
    mode: Mode,
    page_scroll: ReadSignal<Option<ScrollState>>,
) -> Element {
    let mut anchor = use_signal(|| Option::<Rc<MountedData>>::None);

    rsx! {
        section { class: "board {direction}",
            header { class: "board-header",
                h2 { "{title}" }
                code { "{placement_label}" }
            }
            ScrollableView {
                class: "scroll-box",
                tabindex: "0",
                aria_label: title,
                div { class: "scroll-world",
                    div { class: "anchor", onmounted: move |event| anchor.set(Some(event.data())),
                        "Anchor"
                    }
                }
                // The positioning hook captures its options at mount time.
                for mode in [mode] {
                    FloatingPreview {
                        key: "{mode.label()}",
                        anchor,
                        placement,
                        mode,
                        page_scroll,
                    }
                }
            }
        }
    }
}

#[component]
fn FloatingPreview(
    anchor: ReadSignal<Option<Rc<MountedData>>>,
    placement: Placement,
    mode: Mode,
    page_scroll: ReadSignal<Option<ScrollState>>,
) -> Element {
    let mut element = use_signal(|| Option::<Rc<MountedData>>::None);
    let mut scroll_context = use_scroll_context();
    let position = use_placement(
        element,
        anchor,
        FloatingOptions {
            offset: OffsetOptions::new(8.0, 8.0),
            middleware: mode.middleware(),
            padding: 4.0,
            placement,
        },
    );

    use_effect(move || {
        // Moving the outer page also moves each box in viewport coordinates.
        if page_scroll().is_some() {
            spawn(async move { scroll_context.reload().await });
        }
    });

    let pos = position();
    let visibility = if pos.is_ready { "visible" } else { "hidden" };

    rsx! {
        div {
            class: "preview",
            style: "visibility: {visibility}; transform: translate3d({pos.x}px, {pos.y}px, 0)",
            onmounted: move |event| element.set(Some(event.data())),
            strong { "Floating element" }
            span { {mode.label()} }
        }
    }
}
