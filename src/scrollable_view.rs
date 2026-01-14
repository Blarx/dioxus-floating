use std::rc::Rc;

use dioxus::{html::geometry::PixelsVector2D, prelude::*};

use crate::ScrollState;

/// A scrollable container that provides context for floating elements.
///
/// `ScrollableView` is the core component of the library. It tracks its own
/// position, dimensions, and scroll state, providing this data via context
/// to child hooks like `use_placement`.
///
/// # Note on Styles:
/// Ensure you provide height and overflow styles (e.g., `h-full overflow-auto`)
/// via the `class` or `style` props, as the component does not apply them by default.
///
/// # Example
///
/// ```rust,norun
/// use dioxus::prelude::*;
/// use dioxus_floating::{use_scroll_context, ScrollableView, ScrollState};
///
/// #[component]
/// fn MyComponent() -> Element {
///     rsx! {
///         ScrollableView {
///             class: "h-screen w-full overflow-y-auto",
///             on_scroll: move |state: ScrollState| println!("Scrolled to: {}", state.state.y),
///         
///             div { class: "h-[2000px]", "Very long content..." }
///         
///             // Any floating elements inside will be positioned correctly
///             MyDropdown {}
///         }
///     }
/// }
///
/// #[component]
/// fn MyDropdown() -> Element { let ctx = use_scroll_context(); rsx! {} }
/// ```
#[component]
pub fn ScrollableView(
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] style: String,
    children: Element,
    #[props(into)] on_scroll: Option<EventHandler<ScrollState>>,
) -> Element {
    let floating = crate::use_floating();

    let mut scrollable_ref = use_signal(|| Option::<Rc<MountedData>>::None);
    let mut scroll_state = use_signal(|| Option::<ScrollState>::None);

    use_context_provider(move || ScrollableContext {
        scrollable_ref,
        scroll_state,
    });

    rsx! {
        div { id: id, class: class, style: style,
            onmounted: move |evt: MountedEvent| {
                scrollable_ref.set(Some(evt.data.clone()));
                let mounted_data = evt.data.clone();
                spawn(async move {
                    let state = floating.generate_scroll_state_from_mounted(mounted_data).await;
                    scroll_state.set(Some(state));
                });
            },
            onresize: move |evt: ResizeEvent| {
                scroll_state.with_mut(move |sstate| {
                    if let Some(state) = sstate {
                        if let Ok(size) = evt.get_border_box_size() {
                            state.bounds = size;
                        }

                        *sstate = Some(state.to_owned());
                    }
                });
                if let Some(scrollable) = scrollable_ref() {
                    spawn(async move {
                        if let Ok(size) = scrollable.get_scroll_size().await {
                            scroll_state.with_mut(move |sstate| {
                                if let Some(state) = sstate {
                                    state.size = size;
                                    *sstate = Some(state.to_owned());
                                }
                            });
                        }
                    });
                }
            },
            onscroll: move |evt: ScrollEvent| {
                let new_state = floating.generate_scroll_state(evt);
                scroll_state.set(Some(new_state));
                if let Some(cb) = on_scroll { cb.call(new_state); }
            },

            {children}
        }
    }
}

/// Context provided by the [ScrollableView] component.
///
/// It contains reactive signals for the scroll state and a reference to the
/// underlying DOM element, along with methods to programmatically control scrolling.
#[derive(Debug, Clone, Copy)]
pub struct ScrollableContext {
    /// A reactive signal containing the [MountedData] of the scrollable container.
    pub scrollable_ref: Signal<Option<Rc<MountedData>>>,

    /// A reactive signal containing the current [ScrollState] (dimensions, offset, etc.).
    pub scroll_state: Signal<Option<ScrollState>>,
}

impl ScrollableContext {
    /// Forces a re-calculation of the scroll content size and current offset.
    /// Useful when the content inside changes but the container's outer bounds remain the same.
    pub async fn reload(&mut self) {
        if let Some(data) = self.scrollable_ref.peek().as_ref() {
            // Мы используем логику из Floating, которую ты уже написал
            let floating = crate::Floating::default();
            let new_state = floating
                .generate_scroll_state_from_mounted(data.clone())
                .await;

            // Обновляем сигнал
            self.scroll_state.set(Some(new_state));
        }
    }

    /// Programmatically scrolls the container by a given offset.
    ///
    /// # Example
    /// ```rust
    /// use dioxus::prelude::*;
    /// use dioxus::html::geometry::PixelsVector2D;
    /// use dioxus_floating::use_scroll_context;
    ///
    /// #[component]
    /// fn MyComponent() -> Element {
    ///     let ctx = use_scroll_context();
    ///     use_effect(move || {
    ///         spawn(async move {
    ///             ctx.scroll(PixelsVector2D::new(0.0, 100.0), ScrollBehavior::Smooth).await;
    ///         });
    ///     });
    ///     rsx! {}
    /// }
    /// ```
    pub async fn scroll(&self, coordinates: PixelsVector2D, behavior: ScrollBehavior) {
        if let Some(data) = self.scrollable_ref.peek().as_ref() {
            let _ = data.scroll(coordinates, behavior).await;
        }
    }

    /// Scrolls to a specific position (e.g., top or bottom) based on the behavior.
    pub async fn scroll_to(&self, behavior: ScrollBehavior) {
        if let Some(data) = self.scrollable_ref.peek().as_ref() {
            let _ = data.scroll_to(behavior).await;
        }
    }

    /// Scrolls the container using advanced options (like specific element alignment).
    pub async fn scroll_to_with_options(&self, options: ScrollToOptions) {
        if let Some(data) = self.scrollable_ref.peek().as_ref() {
            let _ = data.scroll_to_with_options(options).await;
        }
    }
}
