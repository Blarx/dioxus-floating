use std::rc::Rc;

use dioxus::html::geometry::{ClientPoint, PixelsRect, PixelsSize, PixelsVector2D};
use dioxus::logger::tracing;
use dioxus::prelude::*;

/// The core engine for calculating floating positions.
///
/// `Floating` provides methods to compute the coordinates of elements
/// based on their size, the trigger position, and the boundaries of
/// the scrollable container.
#[derive(Debug, Clone, Copy, Default)]
pub struct Floating;

/// Represents the geometric state of a scrollable container.
#[derive(Debug, Clone, Copy)]
pub struct ScrollState {
    /// Total size of the scrollable content (scrollHeight/scrollWidth).
    pub size: PixelsSize,
    /// Visible dimensions of the container (clientHeight/clientWidth).
    pub bounds: PixelsSize,
    /// Current scroll position (scrollTop/scrollLeft).
    pub state: PixelsVector2D,
}

/// Defines the preferred side and alignment of the floating element relative to its trigger.
#[derive(Debug, Clone, Copy)]
pub enum Placement {
    TopStart,
    TopCenter,
    TopEnd,
    BottomStart,
    BottomCenter,
    BottomEnd,
    LeftStart,
    LeftCenter,
    LeftEnd,
    RightStart,
    RightCenter,
    RightEnd,
}

impl Placement {
    /// Returns `true` if the placement is on the Top or Bottom side.
    pub fn is_vertical(&self) -> bool {
        matches!(
            self,
            Placement::TopStart
                | Placement::TopCenter
                | Placement::TopEnd
                | Placement::BottomStart
                | Placement::BottomCenter
                | Placement::BottomEnd
        )
    }

    /// Returns `true` if the side is Top.
    pub fn is_top(&self) -> bool {
        matches!(
            self,
            Placement::TopEnd | Placement::TopCenter | Placement::TopStart
        )
    }

    /// Returns `true` if the side is Left.
    pub fn is_left(&self) -> bool {
        matches!(
            self,
            Placement::LeftCenter | Placement::LeftEnd | Placement::LeftStart
        )
    }

    /// Returns the [PlacementModifier] (Start, Center, or End) for the current placement.
    pub fn get_modifier(&self) -> PlacementModifier {
        match self {
            &Placement::BottomCenter => PlacementModifier::Center,
            &Placement::LeftCenter => PlacementModifier::Center,
            &Placement::RightCenter => PlacementModifier::Center,
            &Placement::TopCenter => PlacementModifier::Center,
            &Placement::BottomEnd => PlacementModifier::End,
            &Placement::LeftEnd => PlacementModifier::End,
            &Placement::RightEnd => PlacementModifier::End,
            &Placement::TopEnd => PlacementModifier::End,
            &Placement::BottomStart => PlacementModifier::Start,
            &Placement::LeftStart => PlacementModifier::Start,
            &Placement::RightStart => PlacementModifier::Start,
            &Placement::TopStart => PlacementModifier::Start,
        }
    }
}

/// Modifiers that define alignment on the transverse axis.
pub enum PlacementModifier {
    Center,
    Start,
    End,
}

/// Strategic logic used to adjust the floating position when it overflows the viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Middleware {
    /// Flips the element to the opposite side if there isn't enough space (e.g., Top -> Bottom).
    Flip,
    /// Shifts the element along the transverse axis to keep it within the viewport.
    Shift,
}

/// Configuration for the floating position calculation.
#[derive(Debug, Clone)]
pub struct FloatingOptions {
    /// List of [Middleware] strategies to apply.
    pub middleware: Vec<Middleware>,
    /// Distance between the trigger and the floating element in pixels.
    pub offset: f64,
    /// Distance between the floating element and the scrollable container edges.
    pub padding: f64,
    /// The preferred [Placement] strategy.
    pub placement: Placement,
}

impl FloatingOptions {
    /// Returns `true` if the [Middleware::Flip] strategy is enabled.
    pub fn can_flip(&self) -> bool {
        self.middleware.contains(&Middleware::Flip)
    }

    /// Returns `true` if the [Middleware::Shift] strategy is enabled.
    pub fn can_shift(&self) -> bool {
        self.middleware.contains(&Middleware::Shift)
    }
}

impl Default for FloatingOptions {
    /// Returns default options: [Middleware::Flip] and [Middleware::Shift] enabled,
    /// offset: 1.0, padding: 0.0, and [Placement::BottomStart].
    fn default() -> Self {
        FloatingOptions {
            middleware: vec![Middleware::Flip, Middleware::Shift],
            offset: 1_f64,
            padding: 0_f64,
            placement: Placement::BottomStart,
        }
    }
}

impl Floating {
    /// Asynchronously captures the initial [ScrollState] from a mounted element.
    ///
    /// This method is usually called once when the [ScrollableView] is first mounted
    /// or when its underlying DOM element changes. It performs multiple async
    /// JS calls to measure the layout.
    ///
    /// Returns a default state (zeros) if the element is no longer accessible.
    pub async fn generate_scroll_state_from_mounted(&self, data: Rc<MountedData>) -> ScrollState {
        let rect = data.get_client_rect().await;
        let scroll = data.get_scroll_size().await;
        let offset = data.get_scroll_offset().await;

        let size = scroll
            .map(|s| PixelsSize::new(s.width, s.height))
            .unwrap_or(PixelsSize::new(0_f64, 0_f64));
        let bounds = rect
            .map(|r| PixelsSize::new(r.width(), r.height()))
            .unwrap_or(PixelsSize::new(0_f64, 0_f64));
        let state = offset
            .map(|o| PixelsVector2D::new(o.x, o.y))
            .unwrap_or(PixelsVector2D::new(0_f64, 0_f64));

        ScrollState {
            size,
            bounds,
            state,
        }
    }

    /// Synchronously generates a new [ScrollState] from a [ScrollEvent].
    ///
    /// This is a high-performance method designed to be called within the `onscroll`
    /// event handler. It extracts data directly from the event without additional
    /// JS roundtrips.
    ///
    /// # Example
    /// ```rust
    /// onscroll: move |evt| {
    ///     let new_state = floating.generate_scroll_state(evt);
    ///     scroll_state.set(Some(new_state));
    /// }
    /// ```
    pub fn generate_scroll_state(&self, evt: ScrollEvent) -> ScrollState {
        ScrollState {
            size: PixelsSize::new(evt.scroll_width() as f64, evt.scroll_height() as f64),
            bounds: PixelsSize::new(evt.client_width() as f64, evt.client_height() as f64),
            state: PixelsVector2D::new(evt.scroll_left(), evt.scroll_top()),
        }
    }

    /// Calculates the optimal position for a floating element anchored to a specific point (e.g., a mouse click).
    ///
    /// This method treats the input [ClientPoint] as a 1x1 pixel trigger. It is ideal for
    /// context menus where the anchor position is dynamic and precise.
    ///
    /// The returned coordinates (X, Y) are relative to the viewport and are ready
    /// for use with `position: fixed` and `transform: translate3d`.
    pub async fn placement_on_point(
        &self,
        scroll_state: ScrollState,
        scrollable_ref: Rc<MountedData>,
        element_ref: Rc<MountedData>,
        trigger: ClientPoint,
        options: FloatingOptions,
    ) -> (f64, f64) {
        let scrollable_rect = scrollable_ref
            .get_client_rect()
            .await
            .unwrap_or(PixelsRect::new(
                PixelsVector2D::new(0_f64, 0_f64).to_point(),
                scroll_state.bounds,
            ));
        let trigger_rect = PixelsRect::new(
            PixelsVector2D::new(trigger.x, trigger.y).to_point(),
            PixelsSize::new(1_f64, 1_f64),
        );

        match element_ref.get_client_rect().await {
            Ok(element_rect) => {
                self.calculate_placement(scrollable_rect, element_rect, trigger_rect, options)
            }
            Err(_) => (trigger_rect.min_x(), trigger_rect.min_y()),
        }
    }

    /// Calculates the optimal position for a floating element anchored to another DOM element (e.g., a button).
    ///
    /// This method measures the actual dimensions of the trigger element via `get_client_rect()`.
    /// It is designed for standard dropdown menus, tooltips, and popovers where
    /// the floating element needs to align perfectly with its anchor.
    ///
    /// The returned coordinates (X, Y) are viewport-relative.
    pub async fn placement_on_trigger(
        &self,
        scroll_state: ScrollState,
        scrollable_ref: Rc<MountedData>,
        element_ref: Rc<MountedData>,
        trigger_ref: Rc<MountedData>,
        options: FloatingOptions,
    ) -> (f64, f64) {
        let scrollable_rect = scrollable_ref
            .get_client_rect()
            .await
            .unwrap_or(PixelsRect::new(
                PixelsVector2D::new(0_f64, 0_f64).to_point(),
                scroll_state.bounds,
            ));
        let trigger_rect = trigger_ref
            .get_client_rect()
            .await
            .unwrap_or(PixelsRect::new(
                PixelsVector2D::new(0_f64, 0_f64).to_point(),
                PixelsSize::new(1_f64, 1_f64),
            ));

        match element_ref.get_client_rect().await {
            Ok(element_rect) => {
                self.calculate_placement(scrollable_rect, element_rect, trigger_rect, options)
            }
            Err(_) => (trigger_rect.min_x(), trigger_rect.min_y()),
        }
    }

    /// Internal: Computes the initial (ideal) coordinates for the floating element
    /// without considering viewport boundaries or middleware.
    fn compute_base_coords(
        &self,
        element: PixelsRect,
        trigger: PixelsRect,
        options: FloatingOptions,
    ) -> (f64, f64) {
        let x: f64;
        let y: f64;

        // make basic placement element position
        (x, y) = if options.placement.is_vertical() {
            let x = match options.placement.get_modifier() {
                PlacementModifier::Center => {
                    trigger.min_x() + (trigger.width() / 2_f64) - (element.width() / 2_f64)
                }
                PlacementModifier::Start => trigger.min_x(),
                PlacementModifier::End => trigger.max_x() - element.width(),
            };
            let y = if options.placement.is_top() {
                trigger.min_y() - element.height() - options.offset
            } else {
                trigger.max_y() + options.offset
            };
            (x, y)
        } else {
            let x = if options.placement.is_left() {
                trigger.min_x() - element.width() - options.offset
            } else {
                trigger.max_x() + options.offset
            };
            let y = match options.placement.get_modifier() {
                PlacementModifier::Center => {
                    trigger.min_y() + (trigger.height() / 2_f64) - (element.height() / 2_f64)
                }
                PlacementModifier::Start => trigger.min_y(),
                PlacementModifier::End => trigger.max_y() - element.height(),
            };
            (x, y)
        };

        (x, y)
    }

    /// Internal: Adjusts the initial position using the enabled middleware strategies
    /// (Flip and/or Shift) to ensure the element stays within the scrollable area.
    fn apply_middleware(
        &self,
        initial_pos: (f64, f64),
        scrollable: PixelsRect,
        element: PixelsRect,
        trigger: PixelsRect,
        options: FloatingOptions,
    ) -> (f64, f64) {
        let (mut x, mut y) = initial_pos;

        // flip middleware
        if options.can_flip() {
            if options.placement.is_vertical() {
                if options.placement.is_top() && y < scrollable.min_y() {
                    y = trigger.max_y() + options.offset;
                } else if !options.placement.is_top() && y + element.height() > scrollable.max_y() {
                    y = trigger.min_y() - element.height() - options.offset;
                }
            } else {
                if options.placement.is_left() && x < scrollable.min_x() {
                    x = trigger.max_x() + options.offset;
                } else if !options.placement.is_left() && x + element.width() > scrollable.max_x() {
                    x = trigger.min_x() - element.width() - options.offset;
                }
            }
        }
        // shift middleware
        if options.can_shift() {
            if options.placement.is_vertical() {
                // Вычисляем границы: насколько далеко мы можем уйти влево или вправо,
                // чтобы не оторваться от триггера.
                let min_allowed_x = trigger.min_x() - element.width() + options.padding;
                let max_allowed_x = trigger.max_x() - options.padding;

                // 1. Пытаемся вписать в экран (scrollable)
                if x < scrollable.min_x() {
                    x = scrollable.min_x();
                }
                if x + element.width() > scrollable.max_x() {
                    x = scrollable.max_x() - element.width();
                }

                // 2. Но не даем уйти дальше границ триггера
                x = x.clamp(min_allowed_x, max_allowed_x);
            } else {
                let min_allowed_y = trigger.min_y() - element.height() + options.padding;
                let max_allowed_y = trigger.max_y() - options.padding;

                if y < scrollable.min_y() {
                    y = scrollable.min_y();
                }
                if y + element.height() > scrollable.max_y() {
                    y = scrollable.max_y() - element.height();
                }

                y = y.clamp(min_allowed_y, max_allowed_y);
            }
        }

        (x, y)
    }

    /// The main entry point for synchronous position calculation.
    ///
    /// This method takes pre-measured rectangles and applies the full positioning
    /// pipeline: base calculation followed by middleware adjustments.
    ///
    /// It is useful for manual calculations or when you have already obtained
    /// the necessary [PixelsRect] data.
    pub fn calculate_placement(
        &self,
        scrollable: PixelsRect,
        element: PixelsRect,
        trigger: PixelsRect,
        options: FloatingOptions,
    ) -> (f64, f64) {
        let base_pos = self.compute_base_coords(element, trigger, options.clone());
        let final_pos =
            self.apply_middleware(base_pos, scrollable, element, trigger, options.clone());

        tracing::debug!(
            "Calculated for scrollable: {scrollable:?}, element: {element:?}, trigger: {trigger:?}, option: {options:?}"
        );

        final_pos
    }
}
