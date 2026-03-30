//! Studio — unified app workspace for building, previewing, and chatting with the AI agent.
//!
//! Three-panel layout: Cartridges/Files (left) | Preview/Editor (center) | Chat (right)
//! Status bar at bottom shows intelligence metrics in real-time.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::api;

/// App entry — a script endpoint or WASM cartridge.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppEntry {
    slug: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    kind: String, // "script" or "cartridge"
}

/// File entry from the workspace.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct FileEntry {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
    size: Option<u64>,
}

/// Chat message in a session.
#[derive(Clone, Debug)]
struct ChatMsg {
    role: String,
    content: String,
}

/// What the center panel is showing.
#[derive(Clone, Debug, PartialEq)]
enum CenterView {
    Welcome,
    AppPreview(String),            // slug (script — iframe fallback)
    CartridgePreview(String),      // slug (backend WASM — text output)
    InteractivePreview(String),    // slug (interactive WASM — canvas 60fps)
    FileView(String, String),      // path, content
}

/// Studio page — the unified app workspace.
#[component]
pub fn StudioPage() -> impl IntoView {
    // State
    let (apps, set_apps) = create_signal(Vec::<AppEntry>::new());
    let (center, set_center) = create_signal(CenterView::Welcome);
    let (messages, set_messages) = create_signal(Vec::<ChatMsg>::new());
    let (input, set_input) = create_signal(String::new());
    let (sending, set_sending) = create_signal(false);
    let (session_id, set_session_id) = create_signal(None::<String>);
    let (sessions, set_sessions) = create_signal(Vec::<serde_json::Value>::new());
    let (soul_status, set_soul_status) = create_signal(None::<serde_json::Value>);
    let (sys_metrics, set_sys_metrics) = create_signal(None::<serde_json::Value>);
    let (file_tree, set_file_tree) = create_signal(Vec::<FileEntry>::new());
    let (current_path, set_current_path) = create_signal("crates".to_string());
    let (files_expanded, set_files_expanded) = create_signal(false);

    // Fetch apps (scripts + cartridges unified)
    let refresh_apps = move || {
        spawn_local(async move {
            let mut all_apps = Vec::new();

            // Fetch script endpoints
            if let Ok(data) = api::fetch_json("/x").await {
                if let Some(eps) = data.get("endpoints").and_then(|v| v.as_array()) {
                    for ep in eps {
                        let slug = ep
                            .get("slug")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let desc = ep
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        if !slug.is_empty() {
                            all_apps.push(AppEntry {
                                slug,
                                description: desc,
                                kind: "script".to_string(),
                            });
                        }
                    }
                }
            }

            // Fetch WASM cartridges
            if let Ok(data) = api::fetch_json("/c").await {
                if let Some(carts) = data.get("cartridges").and_then(|v| v.as_array()) {
                    for c in carts {
                        let slug = c
                            .get("slug")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let desc = c
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        if !slug.is_empty() {
                            all_apps.push(AppEntry {
                                slug,
                                description: desc,
                                kind: "cartridge".to_string(),
                            });
                        }
                    }
                }
            }

            set_apps.set(all_apps);
        });
    };
    refresh_apps();

    // Fetch chat sessions
    {
        spawn_local(async move {
            if let Ok(data) = api::list_chat_sessions().await {
                if let Some(arr) = data.get("sessions").and_then(|v| v.as_array()) {
                    set_sessions.set(arr.clone());
                }
            }
        });
    }

    // Fetch soul status + system metrics ONCE on load — no polling.
    // Refreshed after each chat message send.
    let refresh_status = move || {
        spawn_local(async move {
            if let Ok(data) = api::fetch_soul_status().await {
                set_soul_status.set(Some(data));
            }
        });
        spawn_local(async move {
            if let Ok(data) = api::fetch_json("/soul/system").await {
                set_sys_metrics.set(Some(data));
            }
        });
    };
    refresh_status();

    // New conversation
    let new_conversation = move |_| {
        set_session_id.set(None);
        set_messages.set(Vec::new());
    };

    // Send chat message
    let send_message = move || {
        let msg = input.get_untracked();
        if msg.trim().is_empty() || sending.get_untracked() {
            return;
        }
        set_sending.set(true);
        set_input.set(String::new());

        set_messages.update(|msgs| {
            msgs.push(ChatMsg {
                role: "user".to_string(),
                content: msg.clone(),
            });
        });

        let sid = session_id.get_untracked();
        let refresh = refresh_apps.clone();
        spawn_local(async move {
            match api::send_soul_chat(&msg, sid.as_deref()).await {
                Ok(resp) => {
                    let reply = resp
                        .get("reply")
                        .and_then(|v| v.as_str())
                        .unwrap_or("(no response)")
                        .to_string();
                    if let Some(new_sid) = resp.get("session_id").and_then(|v| v.as_str()) {
                        set_session_id.set(Some(new_sid.to_string()));
                    }
                    set_messages.update(|msgs| {
                        msgs.push(ChatMsg {
                            role: "assistant".to_string(),
                            content: reply,
                        });
                    });
                    // Reactively refresh apps ONLY if a tool modified endpoints
                    let modified_endpoints = resp
                        .get("tool_executions")
                        .and_then(|v| v.as_array())
                        .map(|execs| {
                            execs.iter().any(|e| {
                                let cmd = e.get("command").and_then(|v| v.as_str()).unwrap_or("");
                                cmd.contains("create_script_endpoint")
                                    || cmd.contains("delete_endpoint")
                                    || cmd.contains("create_cartridge")
                                    || cmd.contains("compile_cartridge")
                                    || cmd.contains("delete_cartridge")
                            })
                        })
                        .unwrap_or(false);
                    if modified_endpoints {
                        refresh();
                    }
                }
                Err(e) => {
                    set_messages.update(|msgs| {
                        msgs.push(ChatMsg {
                            role: "assistant".to_string(),
                            content: format!("Error: {e}"),
                        });
                    });
                }
            }
            set_sending.set(false);
            // Refresh status after chat (no polling — event-driven)
            refresh_status();
        });
    };

    let send_for_key = send_message.clone();
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            send_for_key();
        }
    };

    // Load file tree
    let load_tree = move |path: String| {
        let set_tree = set_file_tree.clone();
        let set_path = set_current_path.clone();
        spawn_local(async move {
            set_path.set(path.clone());
            if let Ok(files) = fetch_file_tree(&path).await {
                set_tree.set(files);
            }
        });
    };

    // Load file content
    let load_file = move |path: String| {
        spawn_local(async move {
            if let Ok(content) = fetch_file_content(&path).await {
                set_center.set(CenterView::FileView(path, content));
            }
        });
    };

    view! {
        <div class="studio">
            // ── Header ──
            <div class="studio-header">
                <div class="studio-header-left">
                    <h2>"Studio"</h2>
                    <button class="btn btn-sm" on:click=new_conversation>"+ New Chat"</button>
                </div>
                <div class="studio-header-right">
                    {move || {
                        let s = soul_status.get();
                        let mode = s.as_ref().and_then(|d| d.get("mode")).and_then(|v| v.as_str()).unwrap_or("--").to_string();
                        let coding = s.as_ref().and_then(|d| d.get("coding_enabled")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let iq = s.as_ref().and_then(|d| d.get("benchmark")).and_then(|b| b.get("opus_iq")).and_then(|v| v.as_str()).unwrap_or("--").to_string();
                        view! {
                            <span class="studio-badge">{mode}</span>
                            <span class="studio-badge">{if coding { "coding" } else { "read-only" }}</span>
                            <span class="studio-badge">"IQ "{iq}</span>
                        }
                    }}
                </div>
            </div>

            // ── Three-panel layout ──
            <div class="studio-layout">

                // ── Left: Apps + Files ──
                <div class="studio-sidebar">
                    <div class="studio-section">
                        <div class="studio-section-header">"Cartridges"</div>
                        {move || {
                            let app_list = apps.get();
                            if app_list.is_empty() {
                                view! {
                                    <div class="studio-empty">
                                        <p>"No cartridges yet"</p>
                                        <p class="studio-hint">"Ask the chat to create a Rust cartridge"</p>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="studio-app-list">
                                        {app_list.iter().map(|app| {
                                            let slug = app.slug.clone();
                                            let kind = app.kind.clone();
                                            let desc = app.description.clone().unwrap_or_default();
                                            let slug_click = slug.clone();
                                            let slug_del = slug.clone();
                                            let refresh_for_del = refresh_apps.clone();
                                            let delete_app = move |ev: web_sys::MouseEvent| {
                                                ev.stop_propagation();
                                                let s = slug_del.clone();
                                                let r = refresh_for_del.clone();
                                                spawn_local(async move {
                                                    let _ = gloo_net::http::Request::delete(
                                                        &format!("/admin/endpoints/script-{}", s)
                                                    ).send().await;
                                                    r();
                                                });
                                            };
                                            let kind_for_click = kind.clone();
                                            view! {
                                                <div
                                                    class="studio-app-item"
                                                    on:click=move |_| {
                                                        if kind_for_click == "cartridge" {
                                                            set_center.set(CenterView::CartridgePreview(slug_click.clone()));
                                                        } else {
                                                            set_center.set(CenterView::AppPreview(slug_click.clone()));
                                                        }
                                                    }
                                                >
                                                    <span class="studio-app-name">{&slug}</span>
                                                    <span class="studio-app-badge">{&kind}</span>
                                                    <button class="studio-app-delete" on:click=delete_app title="Delete">{"\u{00D7}"}</button>
                                                    {(!desc.is_empty()).then(|| view! {
                                                        <span class="studio-app-desc">{&desc}</span>
                                                    })}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </div>

                    // Files (collapsible)
                    <div class="studio-section">
                        <div
                            class="studio-section-header studio-section-toggle"
                            on:click=move |_| {
                                let expanded = !files_expanded.get_untracked();
                                set_files_expanded.set(expanded);
                                if expanded {
                                    load_tree("crates".to_string());
                                }
                            }
                        >
                            {move || if files_expanded.get() { "Files \u{25BE}" } else { "Files \u{25B8}" }}
                        </div>
                        {move || {
                            if !files_expanded.get() {
                                return view! { <span></span> }.into_view();
                            }
                            view! {
                                <div class="studio-file-path">{move || current_path.get()}</div>
                                <div class="studio-file-list">
                                    // Back button
                                    {move || {
                                        let path = current_path.get();
                                        if path != "crates" && path.contains('/') {
                                            let parent = path.rsplit_once('/').map(|(p, _)| p.to_string()).unwrap_or_else(|| "crates".to_string());
                                            let lt = load_tree.clone();
                                            Some(view! {
                                                <div class="studio-file studio-file--dir" on:click=move |_| lt(parent.clone())>
                                                    <span>"\u{2190} .."</span>
                                                </div>
                                            })
                                        } else {
                                            None
                                        }
                                    }}
                                    {move || {
                                        file_tree.get().iter().map(|entry| {
                                            let name = entry.name.clone();
                                            let is_dir = entry.entry_type == "directory";
                                            let full_path = format!("{}/{}", current_path.get_untracked(), name);
                                            let path_for_click = full_path.clone();
                                            let lt = load_tree.clone();
                                            let lf = load_file.clone();
                                            view! {
                                                <div
                                                    class=if is_dir { "studio-file studio-file--dir" } else { "studio-file" }
                                                    on:click=move |_| {
                                                        if is_dir { lt(path_for_click.clone()); }
                                                        else { lf(path_for_click.clone()); }
                                                    }
                                                >
                                                    <span>{if is_dir { "\u{1F4C1} " } else { "" }}</span>
                                                    <span>{&name}</span>
                                                </div>
                                            }
                                        }).collect_view()
                                    }}
                                </div>
                            }.into_view()
                        }}
                    </div>
                </div>

                // ── Center: Preview / Editor / Welcome ──
                <div class="studio-center">
                    {move || {
                        match center.get() {
                            CenterView::Welcome => view! {
                                <div class="studio-welcome">
                                    <h2>"Build something"</h2>
                                    <p>"Select an app to preview, or ask the AI to create one."</p>
                                    <div class="studio-suggestions">
                                        <code>"\"make a snake game\""</code>
                                        <code>"\"build a todo list app\""</code>
                                        <code>"\"create a calculator\""</code>
                                    </div>
                                </div>
                            }.into_view(),
                            CenterView::AppPreview(ref slug) => {
                                let url = format!("/app/{slug}");
                                let slug_for_src = slug.clone();
                                let set_center_for_src = set_center.clone();
                                let view_source = move |_| {
                                    let s = slug_for_src.clone();
                                    let set_c = set_center_for_src.clone();
                                    spawn_local(async move {
                                        let path = format!("/data/endpoints/{s}.sh");
                                        if let Ok(content) = fetch_file_content(&format!("..{}", path)).await {
                                            set_c.set(CenterView::FileView(path, content));
                                        } else {
                                            // Try without prefix
                                            let resp = gloo_net::http::Request::get(&format!("/soul/admin/cat?path=/data/endpoints/{}.sh", s))
                                                .send().await;
                                            if let Ok(r) = resp {
                                                if let Ok(text) = r.text().await {
                                                    set_c.set(CenterView::FileView(format!("/data/endpoints/{s}.sh"), text));
                                                }
                                            }
                                        }
                                    });
                                };
                                view! {
                                    <div class="studio-preview">
                                        <div class="studio-preview-bar">
                                            <span class="studio-preview-url">{&url}</span>
                                            <button class="studio-preview-btn" on:click=view_source>"Source"</button>
                                            <a href={url.clone()} target="_blank" class="studio-preview-open">"Open \u{2197}"</a>
                                        </div>
                                        <iframe
                                            src={url}
                                            class="studio-preview-frame"
                                            sandbox="allow-scripts allow-same-origin"
                                        />
                                    </div>
                                }.into_view()
                            },
                            CenterView::CartridgePreview(ref slug) => {
                                let slug_run = slug.clone();
                                let slug_for_switch = slug.clone();
                                let set_center_for_switch = set_center.clone();
                                let (cartridge_html, set_cartridge_html) = create_signal(String::from("<div class='studio-loading'>Loading cartridge...</div>"));
                                let (cartridge_logs, set_cartridge_logs) = create_signal(Vec::<String>::new());
                                // Detect type then run accordingly
                                spawn_local(async move {
                                    match crate::cartridge_runner::detect_type(&slug_run).await {
                                        Ok((crate::cartridge_runner::CartridgeType::Interactive, _)) => {
                                            // Switch to interactive canvas mode
                                            set_center_for_switch.set(CenterView::InteractivePreview(slug_for_switch));
                                        }
                                        Ok((crate::cartridge_runner::CartridgeType::Backend, _)) => {
                                            // Run as text cartridge
                                            match crate::cartridge_runner::run_cartridge(&slug_run).await {
                                                Ok(output) => {
                                                    set_cartridge_html.set(output.body);
                                                    set_cartridge_logs.set(output.logs);
                                                }
                                                Err(e) => set_cartridge_html.set(format!("<div class='studio-error'><pre>{e}</pre></div>")),
                                            }
                                        }
                                        Err(e) => set_cartridge_html.set(format!("<div class='studio-error'><pre>{e}</pre></div>")),
                                    }
                                });
                                view! {
                                    <div class="studio-preview">
                                        <div class="studio-preview-bar">
                                            <span class="studio-preview-url">"/c/"{slug}" (WASM)"</span>
                                            <a href={format!("/c/{slug}")} target="_blank" class="studio-preview-open">"Open \u{2197}"</a>
                                        </div>
                                        <div class="studio-cartridge-output" inner_html=move || cartridge_html.get() />
                                        {move || {
                                            let logs = cartridge_logs.get();
                                            if logs.is_empty() {
                                                view! { <div /> }.into_view()
                                            } else {
                                                view! {
                                                    <div class="studio-cartridge-logs">
                                                        {logs.iter().map(|l| view! { <div class="studio-log-line">{l}</div> }).collect_view()}
                                                    </div>
                                                }.into_view()
                                            }
                                        }}
                                    </div>
                                }.into_view()
                            },
                            CenterView::InteractivePreview(ref slug) => {
                                let slug_run = slug.clone();
                                let (error_msg, set_error) = create_signal(Option::<String>::None);
                                let canvas_ref = create_node_ref::<leptos::html::Canvas>();

                                // Launch the interactive runtime on mount
                                let slug_for_init = slug.clone();
                                create_effect(move |_| {
                                    let canvas_el = canvas_ref.get();
                                    if canvas_el.is_none() { return; }
                                    let canvas = canvas_el.unwrap();
                                    let slug = slug_for_init.clone();
                                    let set_err = set_error;

                                    spawn_local(async move {
                                        // Fetch and detect (we know it's interactive, but need the bytes)
                                        let bytes = match crate::cartridge_runner::detect_type(&slug).await {
                                            Ok((_, b)) => b,
                                            Err(e) => { set_err.set(Some(e)); return; }
                                        };

                                        let width = 320u32;
                                        let height = 240u32;

                                        // Set canvas dimensions
                                        canvas.set_width(width);
                                        canvas.set_height(height);
                                        let _ = canvas.focus();

                                        let cart = match crate::cartridge_runner::instantiate_interactive(&bytes, width, height).await {
                                            Ok(c) => c,
                                            Err(e) => { set_err.set(Some(e)); return; }
                                        };

                                        let ctx: web_sys::CanvasRenderingContext2d = canvas
                                            .get_context("2d")
                                            .unwrap()
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap();

                                        // Share cartridge across closures
                                        let cart = Rc::new(cart);
                                        let cart_for_loop = cart.clone();
                                        let cart_for_kd = cart.clone();
                                        let cart_for_ku = cart.clone();

                                        // Keyboard handlers
                                        let kd = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                                            ev.prevent_default();
                                            if let Some(ref f) = cart_for_kd.key_down_fn {
                                                let _ = f.call1(&JsValue::undefined(), &JsValue::from(ev.key_code() as i32));
                                            }
                                        });
                                        let ku = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                                            if let Some(ref f) = cart_for_ku.key_up_fn {
                                                let _ = f.call1(&JsValue::undefined(), &JsValue::from(ev.key_code() as i32));
                                            }
                                        });
                                        let _ = canvas.add_event_listener_with_callback("keydown", kd.as_ref().unchecked_ref());
                                        let _ = canvas.add_event_listener_with_callback("keyup", ku.as_ref().unchecked_ref());
                                        kd.forget();
                                        ku.forget();

                                        // requestAnimationFrame loop
                                        let window = web_sys::window().unwrap();
                                        let f: Rc<RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
                                        let g = f.clone();

                                        *g.borrow_mut() = Some(wasm_bindgen::closure::Closure::new(move || {
                                            // Tick
                                            let _ = cart_for_loop.tick_fn.call0(&JsValue::undefined());

                                            // Read framebuffer
                                            let pixels = crate::cartridge_runner::read_framebuffer(&cart_for_loop);

                                            // Blit to canvas
                                            if let Ok(img_data) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                                                wasm_bindgen::Clamped(pixels.as_slice()),
                                                cart_for_loop.width,
                                                cart_for_loop.height,
                                            ) {
                                                let _ = ctx.put_image_data(&img_data, 0.0, 0.0);
                                            }

                                            // Next frame
                                            let win = web_sys::window().unwrap();
                                            let _ = win.request_animation_frame(
                                                f.borrow().as_ref().unwrap().as_ref().unchecked_ref()
                                            );
                                        }));

                                        let _ = window.request_animation_frame(
                                            g.borrow().as_ref().unwrap().as_ref().unchecked_ref()
                                        );
                                    });
                                });

                                view! {
                                    <div class="studio-preview">
                                        <div class="studio-preview-bar">
                                            <span class="studio-preview-url">"/c/"{slug_run}" (Interactive WASM)"</span>
                                        </div>
                                        {move || error_msg.get().map(|e| view! {
                                            <div class="studio-error"><pre>{e}</pre></div>
                                        })}
                                        <div class="studio-canvas-container">
                                            <canvas
                                                node_ref=canvas_ref
                                                class="studio-canvas"
                                                tabindex="0"
                                                width="320"
                                                height="240"
                                            />
                                        </div>
                                    </div>
                                }.into_view()
                            },
                            CenterView::FileView(path, content) => view! {
                                <div class="studio-editor">
                                    <div class="studio-editor-bar">{path}</div>
                                    <pre class="studio-code"><code>{content}</code></pre>
                                </div>
                            }.into_view(),
                        }
                    }}
                </div>

                // ── Right: Chat ──
                <div class="studio-chat">
                    <div class="studio-chat-messages">
                        {move || {
                            let msgs = messages.get();
                            if msgs.is_empty() {
                                view! {
                                    <div class="studio-chat-empty">
                                        <p>"Start a conversation"</p>
                                        <p class="studio-hint">"Tell the AI what to build"</p>
                                    </div>
                                }.into_view()
                            } else {
                                msgs.iter().map(|msg| {
                                    let is_user = msg.role == "user";
                                    let content = msg.content.clone();
                                    view! {
                                        <div class=if is_user { "studio-msg studio-msg--user" } else { "studio-msg studio-msg--ai" }>
                                            <div class="studio-msg-role">{if is_user { "You" } else { "Soul" }}</div>
                                            <div class="studio-msg-content">{content}</div>
                                            {(!is_user).then(|| {
                                                // Per-message feedback state: None, "good", or "bad"
                                                let (feedback_given, set_feedback_given) = create_signal(Option::<String>::None);
                                                view! {
                                                    <div class="studio-msg-feedback">
                                                        {move || {
                                                            match feedback_given.get() {
                                                                Some(ref fb) => view! {
                                                                    <span class={format!("studio-feedback-locked studio-feedback-{fb}")}>{fb.clone()}</span>
                                                                }.into_view(),
                                                                None => view! {
                                                                    <button class="studio-feedback-btn studio-feedback-good" on:click=move |_| {
                                                                        set_feedback_given.set(Some("good".to_string()));
                                                                        spawn_local(async move {
                                                                            let _ = gloo_net::http::Request::post("/soul/admin/reward")
                                                                                .json(&serde_json::json!({"commit_sha": "chat-feedback"}))
                                                                                .unwrap()
                                                                                .send().await;
                                                                        });
                                                                    } title="Good response">"good"</button>
                                                                    <button class="studio-feedback-btn studio-feedback-bad" on:click=move |_| {
                                                                        set_feedback_given.set(Some("bad".to_string()));
                                                                        spawn_local(async move {
                                                                            let _ = gloo_net::http::Request::post("/soul/admin/penalty")
                                                                                .json(&serde_json::json!({"commit_sha": "chat-feedback"}))
                                                                                .unwrap()
                                                                                .send().await;
                                                                        });
                                                                    } title="Bad response">"bad"</button>
                                                                }.into_view(),
                                                            }
                                                        }}
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    }
                                }).collect_view().into_view()
                            }
                        }}
                        {move || sending.get().then(|| view! {
                            <div class="studio-msg studio-msg--ai studio-typing">"Thinking..."</div>
                        })}
                    </div>
                    <div class="studio-chat-input">
                        <textarea
                            placeholder="Tell the soul what to build..."
                            prop:value=move || input.get()
                            on:input=move |ev| set_input.set(event_target_value(&ev))
                            on:keydown=on_keydown
                            rows="2"
                        />
                        <button
                            class="btn btn-primary btn-sm"
                            on:click=move |_| send_message()
                            disabled=move || sending.get()
                        >"Send"</button>
                    </div>
                </div>
            </div>

            // ── Status bar ──
            <div class="studio-statusbar">
                {move || {
                    let s = soul_status.get();
                    let fitness = s.as_ref().and_then(|d| d.get("fitness")).and_then(|f| f.get("total")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let fe = s.as_ref().and_then(|d| d.get("free_energy")).and_then(|f| f.get("F")).and_then(|v| v.as_str()).unwrap_or("--").to_string();
                    let regime = s.as_ref().and_then(|d| d.get("free_energy")).and_then(|f| f.get("regime")).and_then(|v| v.as_str()).unwrap_or("--").to_string();
                    let elo = s.as_ref().and_then(|d| d.get("benchmark")).and_then(|b| b.get("elo")).and_then(|v| v.as_str()).unwrap_or("--").to_string();
                    let psi = s.as_ref().and_then(|d| d.get("colony")).and_then(|c| c.get("psi")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let psi_trend = s.as_ref().and_then(|d| d.get("colony")).and_then(|c| c.get("psi_trend")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let psi_arrow = if psi_trend > 0.001 { "\u{2191}" } else if psi_trend < -0.001 { "\u{2193}" } else { "\u{2192}" };
                    let m = sys_metrics.get();
                    let cpu = m.as_ref().and_then(|d| d.get("cpu_pct")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let mem_pct = m.as_ref().and_then(|d| d.get("mem_pct")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let disk_pct = m.as_ref().and_then(|d| d.get("disk_pct")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let disk_class = if disk_pct > 80.0 { "studio-metric-warn" } else { "" };
                    view! {
                        <span>{format!("Fitness {:.0}%", fitness * 100.0)}</span>
                        <span class="studio-statusbar-sep">"|"</span>
                        <span>{format!("F={fe}")}</span>
                        <span class="studio-statusbar-badge">{regime}</span>
                        <span class="studio-statusbar-sep">"|"</span>
                        <span>{format!("ELO {elo}")}</span>
                        <span class="studio-statusbar-sep">"|"</span>
                        <span>{format!("\u{03A8}={psi:.2}{psi_arrow}")}</span>
                        <span class="studio-statusbar-sep">"|"</span>
                        <span>{format!("CPU {cpu:.0}%")}</span>
                        <span>{format!("RAM {mem_pct:.0}%")}</span>
                        <span class={disk_class}>{format!("Disk {disk_pct:.0}%")}</span>
                    }
                }}
            </div>
        </div>
    }
}

/// Fetch file tree from the admin ls endpoint.
async fn fetch_file_tree(path: &str) -> Result<Vec<FileEntry>, String> {
    let resp = gloo_net::http::Request::get(&format!("/soul/admin/ls?path={}", path))
        .send()
        .await
        .map_err(|e| format!("Failed: {e}"))?;

    if !resp.ok() {
        return Ok(vec![]);
    }

    resp.json::<Vec<FileEntry>>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

/// Fetch file content from the admin cat endpoint.
async fn fetch_file_content(path: &str) -> Result<String, String> {
    let resp = gloo_net::http::Request::get(&format!("/soul/admin/cat?path={}", path))
        .send()
        .await
        .map_err(|e| format!("Failed: {e}"))?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    resp.text().await.map_err(|e| format!("Read error: {e}"))
}
