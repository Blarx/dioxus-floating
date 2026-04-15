use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::{
    document::{Eval, eval},
    prelude::*,
};
use dioxus_floating::{
    FloatingOptions, OffsetOptions, Placement, ScrollableView, use_click_outside, use_placement,
    use_scroll_context,
};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        DemoAssets {}
        ScrollableView {
            class: "h-screen overflow-auto bg-linear-to-br from-slate-100 via-white to-blue-100 text-slate-900",
            TailwindLayoutListener {}
            div {
                class: "mx-auto flex min-h-[140vh] max-w-6xl flex-col gap-8 px-6 py-10",
                DemoHeader {}
                div {
                    class: "grid gap-6 lg:grid-cols-[1.3fr_0.9fr]",
                    ProfileCard {}
                    ActionsCard {}
                }
                div {
                    class: "rounded-box border border-dashed border-base-content/20 bg-base-100/70 p-6 text-sm text-base-content/70 shadow-sm",
                    "Прокрути контейнер, чтобы проверить, что floating-позиционирование остаётся корректным внутри ",
                    code { "ScrollableView" },
                    ". Первый dropdown выровнен по правому краю триггера, второй показывает другой placement и длинный список действий."
                }
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
fn DemoHeader() -> Element {
    rsx! {
        div { class: "space-y-3",
            span { class: "badge badge-soft badge-primary",
                "dioxus-floating examples"
            }
            h1 { class: "text-4xl font-black tracking-tight text-balance",
                "Dropdown examples on top of FlyonUI + Tailwind"
            }
            p { class: "max-w-3xl text-base leading-7 text-slate-600",
                "Эти примеры живут рядом с библиотекой и не зависят от пакета ",
                code { "ui" },
                ". Их можно использовать как smoke-test, когда начнёшь выносить UI-пакеты из воркспейса."
            }
        }
    }
}

#[component]
fn ProfileCard() -> Element {
    rsx! {
        div { class: "card border border-base-content/10 bg-base-100 shadow-xl",
            div { class: "card-body gap-6",
                div { class: "space-y-2",
                    h2 { class: "card-title text-2xl", "Profile dropdown" }
                    p { class: "text-sm text-base-content/70",
                        "Bottom-end placement с широким меню и привычным профайловым триггером."
                    }
                }
                div { class: "flex items-center justify-between gap-4 rounded-box bg-slate-50 p-4",
                    div {
                        h3 { class: "font-semibold", "Alexandr" }
                        p { class: "text-sm text-slate-500", "Maintainer of dioxus-floating" }
                    }
                    DropdownExample {
                        button_class: "btn btn-primary rounded-full px-2 pe-4",
                        menu_class: "min-w-72",
                        placement: Placement::BottomEnd,
                        offset: 12.0,
                        trigger: rsx! {
                            div { class: "flex items-center gap-3",
                                div { class: "avatar avatar-online",
                                    div { class: "w-10 rounded-full bg-primary/15 text-primary",
                                        span { class: "flex h-full w-full items-center justify-center text-sm font-bold",
                                            "AB"
                                        }
                                    }
                                }
                                div { class: "text-left leading-tight",
                                    div { class: "font-semibold", "Workspace profile" }
                                    div { class: "text-xs opacity-70", "Owner access" }
                                }
                            }
                        },
                        li {
                            a { class: "dropdown-item",
                                span { class: "icon-[tabler--layout-dashboard] size-4" }
                                "Dashboard"
                            }
                        }
                        li {
                            a { class: "dropdown-item",
                                span { class: "icon-[tabler--settings] size-4" }
                                "Library settings"
                            }
                        }
                        li {
                            a { class: "dropdown-item",
                                span { class: "icon-[tabler--book] size-4" }
                                "Open README"
                            }
                        }
                        hr { class: "border-base-content/10 -mx-2" }
                        li { class: "dropdown-footer",
                            button { class: "btn btn-error btn-soft btn-block",
                                span { class: "icon-[tabler--logout] size-4" }
                                "Sign out"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActionsCard() -> Element {
    rsx! {
        div { class: "card border border-base-content/10 bg-base-100 shadow-xl",
            div { class: "card-body gap-6",
                div { class: "space-y-2",
                    h2 { class: "card-title text-2xl", "Action dropdown" }
                    p { class: "text-sm text-base-content/70",
                        "Bottom-start placement с другим триггером и длинным меню для проверки shift/flip."
                    }
                }
                div { class: "flex min-h-72 flex-col justify-between rounded-box bg-slate-950 p-5 text-slate-100",
                    div { class: "space-y-2",
                        div { class: "badge badge-soft badge-warning", "Build queue" }
                        h3 { class: "text-xl font-semibold", "Release candidate 0.2.2" }
                        p { class: "text-sm text-slate-300",
                            "Открой меню у карточки ближе к правой границе и проверь, как оно удерживается в пределах scrollable-контейнера."
                        }
                    }
                    div { class: "flex justify-end",
                        DropdownExample {
                            button_class: "btn btn-soft btn-warning",
                            menu_class: "min-w-64",
                            placement: Placement::BottomStart,
                            offset: 10.0,
                            trigger: rsx! {
                                span { class: "inline-flex items-center gap-2",
                                    span { class: "icon-[tabler--dots] size-4" }
                                    "Quick actions"
                                }
                            },
                            li {
                                a { class: "dropdown-item",
                                    span { class: "icon-[tabler--bug] size-4" }
                                    "Open bug triage"
                                }
                            }
                            li {
                                a { class: "dropdown-item",
                                    span { class: "icon-[tabler--flask] size-4" }
                                    "Run regression sandbox"
                                }
                            }
                            li {
                                a { class: "dropdown-item",
                                    span { class: "icon-[tabler--arrows-shuffle] size-4" }
                                    "Re-check placements"
                                }
                            }
                            li {
                                a { class: "dropdown-item",
                                    span { class: "icon-[tabler--package-export] size-4" }
                                    "Prepare crate publish"
                                }
                            }
                            li {
                                a { class: "dropdown-item text-error",
                                    span { class: "icon-[tabler--trash] size-4" }
                                    "Drop draft release"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DropdownExample(
    button_class: String,
    menu_class: String,
    placement: Placement,
    offset: f64,
    trigger: Element,
    children: Element,
) -> Element {
    let dropdown_id = use_stable_id();
    let mut is_open = use_signal(|| false);
    let mut menu_ref = use_signal(|| Option::<Rc<MountedData>>::None);
    let mut trigger_ref = use_signal(|| Option::<Rc<MountedData>>::None);

    use_click_outside(dropdown_id, use_callback(move |_| {
        if is_open() {
            is_open.set(false);
        }
    }));

    use_effect(move || {
        if !is_open() && menu_ref().is_some() {
            menu_ref.set(None);
        }
    });

    let position = use_placement(
        menu_ref,
        trigger_ref,
        FloatingOptions {
            placement,
            offset: OffsetOptions::rect(offset),
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
        div { id: dropdown_id(), class: "dropdown relative inline-flex {open_class}",
            button {
                class: "dropdown-toggle {button_class}",
                onclick: move |evt: MouseEvent| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    is_open.toggle();
                },
                onmounted: move |evt: MountedEvent| trigger_ref.set(Some(evt.data.clone())),
                {trigger}
            }
            if is_open() {
                ul {
                    class: "dropdown-menu block z-50 transition-opacity {menu_class} {menu_visibility_class}",
                    style: menu_style,
                    onclick: move |evt: MouseEvent| {
                        evt.stop_propagation();
                    },
                    onmounted: move |evt: MountedEvent| menu_ref.set(Some(evt.data.clone())),
                    {children}
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

fn use_stable_id() -> ReadSignal<String> {
    use_hook(|| {
        let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Signal::new(format!("floating-demo-{id}")).into()
    })
}
