use std::rc::Rc;

use dioxus::{
    document::{Eval, eval},
    html::geometry::ClientPoint,
    prelude::*,
};
use dioxus_floating::{
    FloatingOptions, OffsetOptions, Placement, ScrollableView, use_placement_on_point,
    use_scroll_context,
};

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        DemoAssets {}
        ScrollableView {
            class: "h-screen overflow-auto bg-linear-to-br from-zinc-950 via-slate-900 to-sky-950 text-white",
            TailwindLayoutListener {}
            div {
                class: "mx-auto flex min-h-[150vh] max-w-6xl flex-col gap-8 px-6 py-10",
                div { class: "space-y-3",
                    span { class: "badge badge-soft badge-info", "context menu" }
                    h1 { class: "text-4xl font-black tracking-tight text-balance",
                        "Right-click dropdown example for dioxus-floating"
                    }
                    p { class: "max-w-3xl text-base leading-7 text-slate-300",
                        "Этот demo повторяет сценарий контекстного меню, но полностью автономен от ",
                        code { "ui" },
                        ". Он удобен как smoke-test для ",
                        code { "use_placement_on_point" },
                        " и для проверки поведения возле границ контейнера."
                    }
                }
                ContextWorkspace {}
            }
        }
    }
}

#[component]
fn DemoAssets() -> Element {
    rsx! {
        document::Stylesheet {
            href: "https://cdn.jsdelivr.net/npm/flyonui@2.4.1/flyonui.css",
        }
        document::Script {
            src: "https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4",
        }
    }
}

#[component]
fn ContextWorkspace() -> Element {
    rsx! {
        div { class: "grid gap-6 lg:grid-cols-[1.1fr_0.9fr]",
            ContextTarget {}
            div { class: "card border border-white/10 bg-white/5 shadow-2xl backdrop-blur",
                div { class: "card-body gap-4",
                    h2 { class: "card-title text-2xl text-white", "What to test" }
                    ul { class: "list-disc space-y-2 pl-5 text-sm text-slate-300",
                        li { "Right click in the middle of the canvas to open the menu at cursor coordinates." }
                        li { "Open the menu near the bottom-right corner and verify flip/shift keep it visible." }
                        li { "Scroll the page and repeat to confirm positioning stays aligned inside ScrollableView." }
                    }
                }
            }
        }
    }
}

#[component]
fn ContextTarget() -> Element {
    let mut is_open = use_signal(|| false);
    let mut click_point = use_signal(|| Option::<ClientPoint>::None);
    let mut menu_ref = use_signal(|| Option::<Rc<MountedData>>::None);

    use_effect(move || {
        if !is_open() {
            click_point.set(None);
            menu_ref.set(None);
        }
    });

    let position = use_placement_on_point(
        menu_ref,
        click_point,
        FloatingOptions {
            placement: Placement::BottomStart,
            offset: OffsetOptions::new(0.0, 10.0),
            ..FloatingOptions::default()
        },
    );

    let menu_style = use_memo(move || {
        position.with(|pos| format!(
            "position: fixed; inset: 0px auto auto 0px; margin: 0px; transform: translate3d({}px, {}px, 0px);",
            pos.x, pos.y,
        ))
    });
    let open_class = use_memo(move || {
        if position().is_ready {
            "open"
        } else {
            ""
        }
    });
    let menu_visibility_class = use_memo(move || {
        if position().is_ready {
            "opacity-100"
        } else {
            "opacity-0"
        }
    });

    rsx! {
        div { class: "relative overflow-hidden rounded-[2rem] border border-white/10 bg-slate-900 shadow-[0_30px_120px_rgba(15,23,42,0.5)]",
            div {
                class: "absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(56,189,248,0.25),transparent_35%),radial-gradient(circle_at_bottom_right,rgba(244,114,182,0.2),transparent_30%)]"
            }
            div {
                class: "relative grid min-h-[32rem] place-items-center p-6",
                oncontextmenu: move |evt: MouseEvent| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    click_point.set(Some(evt.client_coordinates()));
                    is_open.set(true);
                },
                onclick: move |_| {
                    if is_open() {
                        is_open.set(false);
                    }
                },
                div { class: "max-w-md space-y-3 text-center",
                    div { class: "mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-info/20 text-info shadow-inner",
                        span { class: "icon-[tabler--click] size-8" }
                    }
                    h2 { class: "text-2xl font-bold text-white",
                        "Right click anywhere in this panel"
                    }
                    p { class: "text-sm leading-6 text-slate-300",
                        "Меню привязывается к координате курсора через ",
                        code { "use_placement_on_point" },
                        ". Так можно держать рядом с библиотекой реальный пример context-menu trigger."
                    }
                }
            }
            if is_open() && click_point().is_some() {
                div {
                    class: "fixed inset-0 z-40 bg-transparent",
                    onclick: move |_| is_open.set(false),
                }
                ul {
                    class: "dropdown-menu block min-w-72 z-50 transition-opacity {open_class} {menu_visibility_class}",
                    style: menu_style,
                    onmounted: move |evt: MountedEvent| menu_ref.set(Some(evt.data.clone())),
                    oncontextmenu: move |evt: MouseEvent| evt.prevent_default(),
                    onclick: move |evt: MouseEvent| {
                        evt.stop_propagation();
                        is_open.set(false);
                    },
                    li {
                        a { class: "dropdown-item",
                            span { class: "icon-[tabler--copy] size-4" }
                            "Copy message link"
                        }
                    }
                    li {
                        a { class: "dropdown-item",
                            span { class: "icon-[tabler--sparkles] size-4" }
                            "Regenerate completion"
                        }
                    }
                    li {
                        a { class: "dropdown-item",
                            span { class: "icon-[tabler--message-2-share] size-4" }
                            "Move to thread"
                        }
                    }
                    hr { class: "border-base-content/10 -mx-2" }
                    li {
                        a { class: "dropdown-item text-error",
                            span { class: "icon-[tabler--trash] size-4" }
                            "Delete draft"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TailwindLayoutListener() -> Element {
    let mut eval_handle = use_signal(|| Option::<Eval>::None);
    let mut scroll_context = use_scroll_context();

    use_effect(move || {
        spawn(async move {
            let mut eval = eval(
                r#"
                const observer = new MutationObserver(() => {
                    clearTimeout(window.__floating_tailwind_timer);
                    window.__floating_tailwind_timer = setTimeout(() => {
                        dioxus.send("updated");
                    }, 50);
                });

                observer.observe(document.head, { childList: true, subtree: true });

                await dioxus.recv();
                observer.disconnect();
                "#,
            );

            if let Some(old_eval) = eval_handle() {
                let _ = old_eval.send("drop");
            }
            eval_handle.set(Some(eval.clone()));

            while eval.recv::<String>().await.is_ok() {
                scroll_context.reload().await;
            }
        });
    });

    use_drop(move || {
        if let Some(eval) = eval_handle() {
            let _ = eval.send("drop");
        }
    });

    rsx! {}
}
