use std::rc::Rc;

use dioxus::prelude::*;
use dioxus::{html::geometry::ClientPoint, logger::tracing};

mod floating;
mod scrollable_view;

pub use floating::{Floating, FloatingOptions, Middleware, Placement, ScrollState};
pub use scrollable_view::{ScrollableContext, ScrollableView};

/// Returns the global [Floating] engine instance.
///
/// This hook initializes the positioning engine (with default settings)
/// and ensures it persists across component re-renders.
pub fn use_floating() -> Floating {
    use_hook(Floating::default)
}

/// Accesses the nearest [ScrollableContext] provided by a [ScrollableView].
///
/// # Panics
/// This hook will panic if used outside of a [ScrollableView] component.
/// Use `try_use_context::<ScrollableContext>()` if you need a non-panicking version.
pub fn use_scroll_context() -> ScrollableContext {
    use_context::<ScrollableContext>()
}

/// A shorthand hook to access the current [ScrollState] from the context.
///
/// Returns a [Signal] containing the dimensions and scroll offsets of
/// the nearest [ScrollableView].
pub fn use_scroll_state() -> Signal<Option<ScrollState>> {
    let ctx = use_scroll_context();

    ctx.scroll_state
}

/// A shorthand hook to access the [MountedData] of the parent [ScrollableView].
///
/// Useful when you need to programmatically control the scroll container
/// (e.g., calling `scroll_to`) from a child component.
pub fn use_scrollable_ref() -> Signal<Option<Rc<MountedData>>> {
    let ctx = use_scroll_context();

    ctx.scrollable_ref
}

/// The result of a floating position calculation.
///
/// This structure is returned by positioning hooks and contains raw coordinates
/// and a readiness flag. It is designed to be used with `use_memo` to generate
/// custom CSS styles.
#[derive(Debug, Clone, Copy, Default)]
pub struct FloatingResult {
    // Calculated X coordinate (viewport-relative pixels).
    pub x: f64,
    // Calculated Y coordinate (viewport-relative pixels).
    pub y: f64,
    // Use this to toggle visibility (e.g., opacity) to prevent flickering.
    pub is_ready: bool,
}

/// Reactive hook for positioning a floating element relative to a trigger element (anchor).
///
/// This hook automatically finds the nearest [ScrollableView] context to handle
/// scrolling and overflow boundary detection.
///
/// # Behavior
/// - It recalculates the position whenever the trigger, the element itself,
///   or the parent's scroll state changes.
/// - It uses a 1ms delay to ensure the browser has performed a Layout pass
///   before measuring dimensions.
///
/// # Warning
/// This hook must be used within a [ScrollableView] component. If no context
/// is found, it will log a warning and return default (zero) coordinates.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus::html::geometry::PixelsVector2D;
/// use dioxus_floating::{use_placement, FloatingOptions};
///
/// fn MyElement() -> Element {
///     let mut element_ref = use_signal(|| None);
///     let mut trigger_ref = use_signal(|| None);
///     let mut is_opened = use_signal(|| false);
///
///     let placement = use_placement(element_ref, trigger_ref, FloatingOptions::default());
///
///     rsx! {
///         // The trigger element
///         button {
///             onmounted: move |e| trigger_ref.set(Some(e.data.clone())),
///             onclick: move |_| is_opened.toggle(),
///             "Toggle Dropdown"
///         }
///     
///         // The floating element
///         if is_opened() {
///             div {
///                 onmounted: move |e| element_ref.set(Some(e.data.clone())),
///                 // Use is_ready to prevent the element from "jumping" into position
///                 class: if placement().is_ready { "opacity-100" } else { "opacity-0" },
///                 style: "position: fixed; transform: translate3d({placement().x}px, {placement().y}px, 0);",
///                 "I am a dropdown content"
///             }
///         }
///     }
/// }
/// ```
///
/// # Example: Custom Style Generation
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_floating::{use_placement, FloatingOptions};
///
/// #[component]
/// fn MyComponent() -> Element {
///     let el = use_signal(|| None);
///     let tr = use_signal(|| None);
///     let pos = use_placement(el, tr, FloatingOptions::default());
///
///     let style = use_memo(move || {
///         pos.with(|p| format!(
///             "position: fixed; transform: translate3d({}px, {}px, 0); opacity: {};",
///             p.x, p.y, if p.is_ready { 1 } else { 0 }
///         ))
///     });
///     rsx!{}
/// }
/// ```
pub fn use_placement<E, T>(
    // Signal containing the reference to the floating element.
    element_ref: E,
    // Signal containing the reference to the trigger (anchor) element.
    trigger_ref: T,
    // Positioning options including [Placement], [Middleware], and offsets.
    options: FloatingOptions,
) -> ReadSignal<FloatingResult>
where
    E: Into<ReadSignal<Option<Rc<MountedData>>>>,
    T: Into<ReadSignal<Option<Rc<MountedData>>>>,
{
    let element_ref = element_ref.into();
    let trigger_ref = trigger_ref.into();

    let floating = use_floating();
    let mut result = use_signal(FloatingResult::default);

    // context without panic
    let context = match try_use_context::<ScrollableContext>() {
        Some(ctx) => ctx,
        None => {
            tracing::warn!(
                "use_placement hook used outside of ScrollableView. \
                Ensure your component is wrapped in a ScrollableView or provide a ScrollableContext."
            );
            return result.into();
        }
    };

    use_effect(move || {
        let zip = (context.scroll_state)()
            .zip((context.scrollable_ref)())
            .zip(element_ref())
            .zip(trigger_ref());

        if let Some((((scroll_state, scrollable), element), trigger)) = zip {
            let options = options.clone();
            spawn(async move {
                // wait render virtual dom elements
                gloo_timers::future::TimeoutFuture::new(1).await;

                let pos = floating
                    .placement_on_trigger(scroll_state, scrollable, element, trigger, options)
                    .await;

                result.set(FloatingResult {
                    x: pos.0,
                    y: pos.1,
                    is_ready: true,
                });

                tracing::debug!(
                    "Floating placement updated: x={}, y={}, ready=true",
                    pos.0,
                    pos.1
                );
            });
        } else {
            // drop ready flag
            if result.peek().is_ready {
                result.set(FloatingResult::default());
                tracing::debug!("Floating placement reset: ready=false");
            }
        }
    });

    result.into()
}

/// Reactive hook for positioning a floating element relative to a specific point (e.g., mouse click).
///
/// This is specifically designed for context menus or custom popups that appear at
/// a given [ClientPoint]. It automatically subscribes to the nearest [ScrollableView]
/// to handle positioning within a scrollable area.
///
/// # Note on Usage:
/// Unlike `use_placement`, this hook expects a point in viewport coordinates.
/// If you are using this for a context menu, ensure you capture the coordinates
/// from the `MouseEvent`.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_floating::{use_placement_on_point, FloatingOptions};
///
/// #[component]
/// fn MyComponent() -> Element {
///     let mut click_point = use_signal(|| None);
///     let mut element_ref = use_signal(|| None);
///
///     let placement = use_placement_on_point(
///         element_ref,
///         click_point,
///         FloatingOptions::default(),
///     );
///
///     rsx! {
///         div {
///             oncontextmenu: move |e| {
///                 e.prevent_default();
///                 click_point.set(Some(e.client_coordinates()));
///             },
///             "Right click here to open menu"
///         }
///     
///         // Render the element as soon as we have a target point
///         if click_point().is_some() {
///             div {
///                 onmounted: move |e| element_ref.set(Some(e.data.clone())),
///                 // Keep it invisible until positioning is calculated
///                 class: if placement().is_ready { "opacity-100" } else { "opacity-0" },
///                 style: "position: fixed; transform: translate3d({placement().x}px, {placement().y}px, 0);",
///                 "Context Menu Content"
///             }
///         }
///     }
/// }
/// ```
///
/// # Example: Custom Style Generation
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_floating::{use_placement_on_point, FloatingOptions};
///
/// #[component]
/// fn MyComponent() -> Element {
///     let el = use_signal(|| None);
///     let mut click = use_signal(|| None);
///     let pos = use_placement_on_point(el, click, FloatingOptions::default());
///     let style = use_memo(move || {
///         pos.with(|p| format!(
///             "position: fixed; transform: translate3d({}px, {}px, 0); opacity: {};",
///             p.x, p.y, if p.is_ready { 1 } else { 0 }
///         ))
///     });
///     rsx! {
///         button {
///             onclick: move |evt: MouseEvent| { click.set(Some(evt.client_coordinates())) }
///         }
///     }
/// }
/// ```
pub fn use_placement_on_point<E, T>(
    element_ref: E,
    trigger_point: T,
    options: FloatingOptions,
) -> ReadSignal<FloatingResult>
where
    E: Into<ReadSignal<Option<Rc<MountedData>>>>,
    T: Into<ReadSignal<Option<ClientPoint>>>,
{
    let element_ref = element_ref.into();
    let trigger_point = trigger_point.into();
    let floating = use_floating();
    let mut result = use_signal(FloatingResult::default);
    // context without panic
    let context = match try_use_context::<ScrollableContext>() {
        Some(ctx) => ctx,
        None => {
            tracing::warn!(
                "use_placement hook used outside of ScrollableView. \
                Ensure your component is wrapped in a ScrollableView or provide a ScrollableContext."
            );
            return result.into();
        }
    };

    use_effect(move || {
        let zip = (context.scroll_state)()
            .zip((context.scrollable_ref)())
            .zip(element_ref())
            .zip(trigger_point());

        if let Some((((scroll_state, scrollable), element), trigger)) = zip {
            let options = options.clone();
            spawn(async move {
                // wait render virtual dom elements
                gloo_timers::future::TimeoutFuture::new(1).await;

                let pos = floating
                    .placement_on_point(scroll_state, scrollable, element, trigger, options)
                    .await;

                result.set(FloatingResult {
                    x: pos.0,
                    y: pos.1,
                    is_ready: true,
                });

                tracing::debug!(
                    "Floating placement updated: x={}, y={}, ready=true",
                    pos.0,
                    pos.1
                );
            });
        } else {
            // drop ready flag
            if result.peek().is_ready {
                result.set(FloatingResult::default());
                tracing::debug!("Floating placement reset: ready=false");
            }
        }
    });

    result.into()
}
