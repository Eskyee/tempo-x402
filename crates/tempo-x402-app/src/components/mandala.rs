use crate::api;
use crate::WalletState;
use gloo_timers::callback::Interval;
use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::wallet_panel::WalletButtons;

#[derive(Clone, Debug)]
struct SoulEventMsg {
    code: String,
    message: String,
}

/// The Cell — biological visualization of the cognitive architecture.
/// Each system is a distinct organelle. Events flow as vesicles.
#[component]
pub fn Mandala() -> impl IntoView {
    let (wallet, set_wallet) =
        expect_context::<(ReadSignal<WalletState>, WriteSignal<WalletState>)>();

    let (soul, set_soul) = create_signal(None::<serde_json::Value>);
    let (info, set_info) = create_signal(None::<serde_json::Value>);
    let (system, set_system) = create_signal(None::<serde_json::Value>);
    let (panel_open, set_panel_open) = create_signal(false);
    let (clone_loading, set_clone_loading) = create_signal(false);
    let (clone_result, set_clone_result) = create_signal(None::<Result<String, String>>);
    let (events, set_events) = create_signal(Vec::<SoulEventMsg>::new());
    let (pulses, set_pulses) = create_signal(std::collections::HashMap::<String, f64>::new());

    // Fetch state
    let fetch_all = move || {
        spawn_local(async move {
            let base = api::gateway_base_url();
            if let Ok(resp) = gloo_net::http::Request::get(&format!("{}/instance/info", base)).send().await {
                if resp.ok() { if let Ok(d) = resp.json::<serde_json::Value>().await { set_info.set(Some(d)); } }
            }
            if let Ok(data) = api::fetch_soul_status().await { set_soul.set(Some(data)); }
            if let Ok(resp) = gloo_net::http::Request::get(&format!("{}/soul/system", base)).send().await {
                if resp.ok() { if let Ok(d) = resp.json::<serde_json::Value>().await { set_system.set(Some(d)); } }
            }
        });
    };
    fetch_all();
    let interval = Interval::new(10_000, move || { fetch_all(); });
    on_cleanup(move || drop(interval));

    // SSE
    {
        let base = api::gateway_base_url().to_string();
        spawn_local(async move {
            let url = format!("{}/soul/events/stream", base);
            let es = match web_sys::EventSource::new(&url) { Ok(es) => es, Err(_) => return };
            let on_msg = Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
                let data_str = ev.data().as_string().unwrap_or_default();
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data_str) {
                    let code = parsed.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let message = parsed.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if !code.is_empty() && code != "heartbeat" {
                        set_events.update(|e| { e.push(SoulEventMsg { code: code.clone(), message }); if e.len() > 20 { e.drain(..e.len()-20); } });
                        set_pulses.update(|p| { p.insert(code, js_sys::Date::now()); });
                    }
                }
            });
            es.add_event_listener_with_callback("soul_event", on_msg.as_ref().unchecked_ref()).ok();
            on_msg.forget();
        });
    }

    // Center of cell
    let cx = 400.0f64;
    let cy = 400.0f64;

    view! {
        <div class="mandala-container">
            <svg viewBox="0 0 800 800" class="mandala-svg" preserveAspectRatio="xMidYMid meet">
                <defs>
                    <filter id="glow">
                        <feGaussianBlur stdDeviation="3" result="b"/>
                        <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
                    </filter>
                    <filter id="glow-strong">
                        <feGaussianBlur stdDeviation="6" result="b"/>
                        <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
                    </filter>
                </defs>

                // ═══ CELL MEMBRANE (Free Energy) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let fe = s.get("free_energy");
                    let regime = fe.and_then(|f| f.get("regime")).and_then(|v| v.as_str()).unwrap_or("EXPLOIT");
                    let f_val = fe.and_then(|f| f.get("F")).and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.3);
                    let (membrane_color, membrane_glow) = match regime {
                        "EXPLORE" => ("#00e5ff", "rgba(0,229,255,0.08)"),
                        "LEARN" => ("#b388ff", "rgba(179,136,255,0.08)"),
                        "EXPLOIT" => ("#00ff41", "rgba(0,255,65,0.06)"),
                        "ANOMALY" => ("#ff1744", "rgba(255,23,68,0.1)"),
                        _ => ("#00ff41", "rgba(0,255,65,0.06)"),
                    };
                    let thickness = 2.0 + f_val * 4.0; // thicker = more surprise
                    // Wavy membrane path (phospholipid bilayer look)
                    let outer = membrane_path(cx, cy, 310.0, 12, 6.0);
                    let inner = membrane_path(cx, cy, 300.0, 12, 4.0);
                    view! {
                        // Background fill
                        <ellipse cx=cx.to_string() cy=cy.to_string() rx="310" ry="310"
                            fill=membrane_glow stroke="none"/>
                        // Outer membrane
                        <path d=outer fill="none" stroke=membrane_color
                            stroke-width=thickness.to_string() opacity="0.5"/>
                        // Inner membrane
                        <path d=inner fill="none" stroke=membrane_color
                            stroke-width=(thickness * 0.6).to_string() opacity="0.3"/>
                        // Regime label on membrane
                        <text x=cx.to_string() y="75" text-anchor="middle"
                            class="mandala-text-system" fill=membrane_color opacity="0.5">
                            {format!("F={:.3} {}", f_val, regime)}
                        </text>
                    }
                }}

                // ═══ CYTOSKELETON (Autonomy — fine lines connecting organelles) ═══
                {move || {
                    let p = pulses.get();
                    let now = js_sys::Date::now();
                    // Structural lines between major organelles
                    let connections = [
                        (cx, cy, cx - 140.0, cy + 100.0, "brain"),    // nucleus→mitochondria
                        (cx, cy, cx + 150.0, cy + 80.0, "hivemind"),  // nucleus→golgi
                        (cx - 140.0, cy + 100.0, cx + 60.0, cy - 20.0, "codegen"), // mito→ER/ribosomes
                        (cx, cy, cx - 60.0, cy + 180.0, "evaluation"), // nucleus→lysosome
                    ];
                    connections.iter().map(|(x1, y1, x2, y2, prefix)| {
                        let intensity = pulse_intensity(&p, prefix, now);
                        let opacity = 0.06 + intensity * 0.3;
                        let color = if intensity > 0.2 { "#00ff41" } else { "#1a1a2e" };
                        view! {
                            <line x1=x1.to_string() y1=y1.to_string()
                                x2=x2.to_string() y2=y2.to_string()
                                stroke=color stroke-width="0.8" opacity=opacity.to_string()
                                stroke-dasharray="4 8"/>
                        }
                    }).collect::<Vec<_>>()
                }}

                // ═══ ENDOPLASMIC RETICULUM (Cortex — folded membrane network) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let c = s.get("cortex");
                    let exp = c.and_then(|c| c.get("total_experiences")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let acc = c.and_then(|c| c.get("prediction_accuracy")).and_then(|v| v.as_str())
                        .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok()).unwrap_or(0.0) / 100.0;
                    let folds = (3 + (exp as f64).sqrt() as usize).min(8);
                    let opacity = 0.2 + acc * 0.4;
                    // ER folds extending from nucleus
                    let er_paths: Vec<String> = (0..folds).map(|i| {
                        let y_offset = -80.0 + (i as f64) * 25.0;
                        let amplitude = 15.0 + (i as f64) * 3.0;
                        let x_start = cx - 30.0;
                        let x_end = cx + 160.0;
                        format!("M {:.0},{:.0} C {:.0},{:.0} {:.0},{:.0} {:.0},{:.0}",
                            x_start, cy + y_offset,
                            x_start + 50.0, cy + y_offset - amplitude,
                            x_end - 50.0, cy + y_offset + amplitude,
                            x_end, cy + y_offset)
                    }).collect();
                    view! {
                        {er_paths.iter().map(|d| {
                            let d = d.clone();
                            view! {
                                <path d=d fill="none" stroke="#00e5ff"
                                    stroke-width="1.5" opacity=opacity.to_string()/>
                            }
                        }).collect::<Vec<_>>()}
                        <text x=(cx + 170.0).to_string() y=(cy - 40.0).to_string()
                            class="mandala-text-tiny" fill="#00e5ff" opacity="0.5">
                            {format!("cortex {}exp {:.0}%", exp, acc * 100.0)}
                        </text>
                    }
                }}

                // ═══ RIBOSOMES on ER (Codegen — small dots along ER surface) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let cg = s.get("codegen");
                    let sols = cg.and_then(|c| c.get("solutions_stored")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let steps = cg.and_then(|c| c.get("model_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let can = cg.and_then(|c| c.get("can_generate")).and_then(|v| v.as_bool()).unwrap_or(false);
                    let color = if can { "#00ff41" } else if sols > 0 { "#ffa000" } else { "#ff1744" };
                    let n = (sols as usize).min(15);
                    // Place along ER folds
                    (0..n).map(|i| {
                        let t = i as f64 / n.max(1) as f64;
                        let rx = cx + 20.0 + t * 130.0;
                        let ry = cy - 60.0 + (t * 5.0).sin() * 20.0;
                        let r = 2.0 + (steps as f64 / 500.0).min(2.0);
                        view! {
                            <circle cx=rx.to_string() cy=ry.to_string() r=r.to_string()
                                fill=color opacity="0.7"/>
                        }
                    }).collect::<Vec<_>>()
                }}

                // ═══ NUCLEUS (Synthesis — double-membrane oval) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let synth = s.get("synthesis");
                    let state = synth.and_then(|s| s.get("state")).and_then(|v| v.as_str()).unwrap_or("--");
                    let nucleus_color = match state {
                        "coherent" | "exploiting" => "#00ff41",
                        "exploring" => "#00e5ff",
                        "conflicted" => "#ffa000",
                        "stuck" => "#ff1744",
                        _ => "#5a6a5a",
                    };
                    let role = s.get("role");
                    let psi = role.and_then(|r| r.get("psi")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                    // Genesis helices inside nucleus
                    let gen = s.get("genesis");
                    let generation = gen.and_then(|g| g.get("generation")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let templates = gen.and_then(|g| g.get("templates")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let helix1 = dna_helix(cx - 25.0, cy - 15.0, 50.0, 8.0, 0.0);
                    let helix2 = dna_helix(cx - 25.0, cy + 10.0, 50.0, 6.0, 1.5);

                    view! {
                        // Outer nuclear membrane
                        <ellipse cx=cx.to_string() cy=cy.to_string() rx="75" ry="60"
                            fill="none" stroke=nucleus_color stroke-width="2" opacity="0.6"
                            filter="url(#glow)"/>
                        // Inner nuclear membrane
                        <ellipse cx=cx.to_string() cy=cy.to_string() rx="68" ry="53"
                            fill="rgba(0,20,10,0.5)" stroke=nucleus_color stroke-width="1" opacity="0.3"/>
                        // Genesis DNA helices
                        <path d=helix1 fill="none" stroke="#b388ff" stroke-width="1.2" opacity="0.5"/>
                        <path d=helix2 fill="none" stroke="#b388ff" stroke-width="1.0" opacity="0.35"/>
                        // Psi at center
                        <text x=cx.to_string() y=(cy - 25.0).to_string()
                            text-anchor="middle" class="mandala-text-psi" fill=nucleus_color>
                            {format!("\u{03A8} {:.3}", psi)}
                        </text>
                        // State label
                        <text x=cx.to_string() y=(cy + 35.0).to_string()
                            text-anchor="middle" class="mandala-text-tiny" fill=nucleus_color opacity="0.6">
                            {state.to_string()}
                        </text>
                        // Genesis info
                        <text x=cx.to_string() y=(cy + 45.0).to_string()
                            text-anchor="middle" class="mandala-text-tiny" fill="#b388ff" opacity="0.4">
                            {format!("gen{} {}tmpl", generation, templates)}
                        </text>
                    }
                }}

                // ═══ MITOCHONDRIA (Brain — bean shape with cristae) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let b = s.get("brain");
                    let loss = b.and_then(|b| b.get("running_loss")).and_then(|v| v.as_f64()).unwrap_or(1.0);
                    let steps = b.and_then(|b| b.get("train_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let health = 1.0 - loss.min(1.0);
                    let brightness = 0.3 + health * 0.5;
                    let p = pulses.get();
                    let pulse = pulse_intensity(&p, "brain", js_sys::Date::now());
                    let opacity = brightness + pulse * 0.3;
                    let mx = cx - 140.0;
                    let my = cy + 100.0;
                    // Bean shape
                    let bean = format!(
                        "M {},{} C {},{} {},{} {},{} C {},{} {},{} {},{}",
                        mx - 35.0, my,
                        mx - 35.0, my - 30.0, mx + 35.0, my - 25.0, mx + 35.0, my,
                        mx + 35.0, my + 30.0, mx - 35.0, my + 25.0, mx - 35.0, my
                    );
                    // Cristae (internal folds)
                    let n_cristae = (3 + steps / 5000).min(7) as usize;
                    let cristae: Vec<String> = (0..n_cristae).map(|i| {
                        let y = my - 15.0 + (i as f64) * 7.0;
                        format!("M {:.0},{:.0} Q {:.0},{:.0} {:.0},{:.0}",
                            mx - 20.0, y, mx, y - 4.0, mx + 20.0, y)
                    }).collect();
                    let filter = if pulse > 0.2 { "url(#glow-strong)" } else { "url(#glow)" };
                    view! {
                        <path d=bean fill="rgba(0,40,20,0.4)" stroke="#00ff41"
                            stroke-width="1.5" opacity=opacity.to_string() filter=filter/>
                        {cristae.iter().map(|d| {
                            let d = d.clone();
                            view! { <path d=d fill="none" stroke="#00ff41" stroke-width="0.8" opacity=(opacity * 0.6).to_string()/> }
                        }).collect::<Vec<_>>()}
                        <text x=mx.to_string() y=(my + 35.0).to_string()
                            text-anchor="middle" class="mandala-text-tiny" fill="#00ff41" opacity="0.5">
                            {format!("brain {}K L={:.2}", steps/1000, loss)}
                        </text>
                    }
                }}

                // ═══ GOLGI APPARATUS (Hivemind — stacked cisternae) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let h = s.get("hivemind");
                    let trails = h.and_then(|h| h.get("total_trails")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let deposits = h.and_then(|h| h.get("total_deposits")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let n_stacks = (3 + trails / 10).min(6) as usize;
                    let intensity = (deposits as f64 / 50.0).min(1.0);
                    let gx = cx + 150.0;
                    let gy = cy + 80.0;
                    let cisternae: Vec<String> = (0..n_stacks).map(|i| {
                        let y = gy - 15.0 + (i as f64) * 8.0;
                        let curve = 12.0 - (i as f64) * 1.5;
                        format!("M {:.0},{:.0} Q {:.0},{:.0} {:.0},{:.0}",
                            gx - 30.0, y, gx, y - curve, gx + 30.0, y)
                    }).collect();
                    view! {
                        {cisternae.iter().enumerate().map(|(i, d)| {
                            let d = d.clone();
                            let op = 0.25 + intensity * 0.4 - (i as f64) * 0.03;
                            view! { <path d=d fill="none" stroke="#ffa000" stroke-width="2" opacity=op.to_string()/> }
                        }).collect::<Vec<_>>()}
                        <text x=gx.to_string() y=(gy + 30.0).to_string()
                            text-anchor="middle" class="mandala-text-tiny" fill="#ffa000" opacity="0.5">
                            {format!("hive {}t {}d", trails, deposits)}
                        </text>
                    }
                }}

                // ═══ LYSOSOMES (Evaluation — small dense circles) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let e = s.get("evaluation");
                    let records = e.and_then(|e| e.get("total_records")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let health = (records as f64 / 100.0).min(1.0);
                    let lx = cx - 60.0;
                    let ly = cy + 190.0;
                    let n = (1 + records / 20).min(4) as usize;
                    view! {
                        {(0..n).map(|i| {
                            let x = lx + (i as f64) * 16.0;
                            let r = 5.0 + health * 3.0;
                            view! {
                                <circle cx=x.to_string() cy=ly.to_string() r=r.to_string()
                                    fill="rgba(255,23,68,0.15)" stroke="#ff1744"
                                    stroke-width="1" opacity=(0.3 + health * 0.4).to_string()/>
                            }
                        }).collect::<Vec<_>>()}
                        <text x=lx.to_string() y=(ly + 16.0).to_string()
                            class="mandala-text-tiny" fill="#ff1744" opacity="0.4">
                            {format!("eval {}rec", records)}
                        </text>
                    }
                }}

                // ═══ METRICS (outside cell membrane) ═══
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let d = info.get().unwrap_or_default();
                    let bench = s.get("benchmark");
                    let iq = bench.and_then(|b| b.get("opus_iq")).and_then(|v| v.as_str()).unwrap_or("--");
                    let elo = bench.and_then(|b| b.get("elo_rating")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let pass = bench.and_then(|b| b.get("pass_at_1")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let accel = s.get("acceleration");
                    let alpha: f64 = accel.and_then(|a| a.get("alpha")).and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let regime = accel.and_then(|a| a.get("regime")).and_then(|v| v.as_str()).unwrap_or("STALLED");
                    let (a_color, a_sym) = match regime {
                        "ACCELERATING" => ("#00ff41", "\u{25B2}"),
                        "CRUISING" => ("#00e5ff", "\u{25C6}"),
                        "DECELERATING" => ("#ff1744", "\u{25BC}"),
                        _ => ("#5a6a5a", "\u{25CB}"),
                    };
                    let fitness = d.get("fitness").and_then(|f| f.get("total")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let cycles = s.get("total_cycles").and_then(|v| v.as_u64()).unwrap_or(0);
                    let mode = s.get("mode").and_then(|v| v.as_str()).unwrap_or("--");
                    let active = s.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                    let sys = system.get().unwrap_or_default();
                    let cpu = sys.get("cpu_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);

                    view! {
                        // IQ — top center, prominent
                        <text x=cx.to_string() y="30" text-anchor="middle"
                            class="mandala-text-iq" fill=a_color>
                            {format!("IQ {} | ELO {:.0} | {:.1}%", iq, elo, pass)}
                        </text>
                        // Alpha — below IQ
                        <text x=cx.to_string() y="48" text-anchor="middle"
                            class="mandala-text-alpha" fill=a_color>
                            {format!("{} \u{03B1}={:+.4} {}", a_sym, alpha, regime)}
                        </text>
                        // Bottom: status
                        <text x="16" y="780" class="mandala-text-label" fill="#3a4a3a">
                            {format!("{} | cycle {} | fitness {:.0}% | cpu {:.0}%", mode, cycles, fitness * 100.0, cpu)}
                        </text>
                        <text x="784" y="780" text-anchor="end" class="mandala-text-label" fill="#1a1a2e">
                            {concat!("v", env!("CARGO_PKG_VERSION"))}
                        </text>
                        <circle cx="10" cy="776" r="3" fill=if active { "#00ff41" } else { "#ff1744" }/>
                    }
                }}

                // ═══ EVENT LOG (vesicle trail — bottom left inside membrane) ═══
                {move || {
                    let evts = events.get();
                    evts.iter().rev().take(6).enumerate().map(|(i, evt)| {
                        let y = 710.0 - (i as f64) * 11.0;
                        let opacity = 0.6 - (i as f64) * 0.08;
                        let color = event_color(&evt.code);
                        let msg: String = evt.message.chars().take(50).collect();
                        view! {
                            <text x="110" y=y.to_string() class="mandala-text-tiny" fill=color opacity=opacity.to_string()>
                                {format!("[{}] {}", event_abbr(&evt.code), msg)}
                            </text>
                        }
                    }).collect::<Vec<_>>()
                }}
            </svg>

            // ── Control panel ──
            <div class="mandala-controls">
                <button class="mandala-toggle" on:click=move |_| set_panel_open.update(|v| *v = !*v)>
                    {move || if panel_open.get() { "\u{2715}" } else { "\u{2630}" }}
                </button>
                <Show when=move || panel_open.get() fallback=|| ()>
                    <div class="mandala-panel">
                        <div class="mandala-panel-section">
                            <div class="mandala-panel-label">"ACCOUNT"</div>
                            <WalletButtons wallet=wallet set_wallet=set_wallet />
                        </div>
                        {move || {
                            let w = wallet.get();
                            if !w.connected { return view! { <div></div> }.into_view(); }
                            let addr = w.address.unwrap_or_default();
                            let short = if addr.len() > 10 { format!("{}...{}", &addr[..6], &addr[addr.len()-4..]) } else { addr };
                            view! { <div class="mandala-panel-section"><div style="font-size:10px;color:var(--text-dim)">{short}</div></div> }.into_view()
                        }}
                        {move || {
                            let d = info.get().unwrap_or_default();
                            let avail = d.get("clone_available").and_then(|v| v.as_bool()).unwrap_or(false);
                            let price = d.get("clone_price").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();
                            if !avail { return view! { <div></div> }.into_view(); }
                            let do_clone = move |_: web_sys::MouseEvent| {
                                if clone_loading.get() { return; }
                                let w = wallet.get();
                                if !w.connected { return; }
                                set_clone_loading.set(true);
                                set_clone_result.set(None);
                                spawn_local(async move {
                                    match api::clone_instance(&w).await {
                                        Ok(r) => set_clone_result.set(Some(Ok(format!("Clone {} at {}", r.instance_id.unwrap_or_default(), r.url.unwrap_or_default())))),
                                        Err(e) => set_clone_result.set(Some(Err(e))),
                                    }
                                    set_clone_loading.set(false);
                                });
                            };
                            view! {
                                <div class="mandala-panel-section">
                                    <div class="mandala-panel-label">"CLONE"</div>
                                    <button class="btn btn-primary" on:click=do_clone disabled=move || clone_loading.get() || !wallet.get().connected>
                                        {move || if clone_loading.get() { "Cloning..." } else { "Clone Node" }}
                                    </button>
                                    <div style="font-size:9px;color:var(--text-muted);margin-top:2px">{format!("${}", price)}</div>
                                    {move || clone_result.get().map(|r| match r {
                                        Ok(m) => view! { <div style="font-size:9px;color:var(--green);margin-top:4px">{m}</div> }.into_view(),
                                        Err(e) => view! { <div style="font-size:9px;color:var(--red);margin-top:4px">{e}</div> }.into_view(),
                                    })}
                                </div>
                            }.into_view()
                        }}
                        <div class="mandala-panel-section">
                            <div class="mandala-panel-label">"NAVIGATE"</div>
                            <a href="/dashboard" class="mandala-nav-link">"Dashboard"</a>
                            <a href="/studio" class="mandala-nav-link">"Studio"</a>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

// ── SVG generators ──

/// Wavy membrane path (elliptical with perturbations)
fn membrane_path(cx: f64, cy: f64, radius: f64, bumps: usize, amplitude: f64) -> String {
    let n = bumps * 8; // smoothness
    let mut d = String::new();
    for i in 0..=n {
        let t = (i as f64 / n as f64) * std::f64::consts::TAU;
        let wobble = (t * bumps as f64).sin() * amplitude;
        let r = radius + wobble;
        let x = cx + r * t.cos();
        let y = cy + r * 0.95 * t.sin(); // slightly squashed
        if i == 0 { d.push_str(&format!("M {:.1},{:.1}", x, y)); }
        else { d.push_str(&format!(" L {:.1},{:.1}", x, y)); }
    }
    d.push_str(" Z");
    d
}

/// DNA double helix path (sinusoidal)
fn dna_helix(x: f64, y: f64, width: f64, amplitude: f64, phase: f64) -> String {
    let steps = 20;
    let mut d1 = String::new();
    let mut d2 = String::new();
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let px = x + t * width;
        let py1 = y + (t * std::f64::consts::TAU * 2.0 + phase).sin() * amplitude;
        let py2 = y + (t * std::f64::consts::TAU * 2.0 + phase + std::f64::consts::PI).sin() * amplitude;
        if i == 0 {
            d1.push_str(&format!("M {:.1},{:.1}", px, py1));
            d2.push_str(&format!("M {:.1},{:.1}", px, py2));
        } else {
            d1.push_str(&format!(" L {:.1},{:.1}", px, py1));
            d2.push_str(&format!(" L {:.1},{:.1}", px, py2));
        }
    }
    format!("{} {}", d1, d2)
}

fn pulse_intensity(pulses: &std::collections::HashMap<String, f64>, prefix: &str, now: f64) -> f64 {
    let last = pulses.iter()
        .filter(|(code, _)| code.starts_with(prefix))
        .map(|(_, ts)| *ts)
        .fold(0.0f64, f64::max);
    if last == 0.0 { return 0.0; }
    (1.0 - ((now - last) / 10_000.0)).max(0.0)
}

fn event_color(code: &str) -> &'static str {
    if code.starts_with("brain") { "#00ff41" }
    else if code.starts_with("transformer") { "#00e5ff" }
    else if code.starts_with("codegen") { "#ffa000" }
    else if code.starts_with("plan") { "#b388ff" }
    else if code.starts_with("benchmark") { "#00ff41" }
    else if code.starts_with("peer") { "#00e5ff" }
    else { "#3a4a3a" }
}

fn event_abbr(code: &str) -> &'static str {
    if code.starts_with("brain.trained") { "MITO" }
    else if code.starts_with("transformer") { "NUCL" }
    else if code.starts_with("codegen") { "RIBO" }
    else if code.starts_with("plan.step.completed") { "ER+" }
    else if code.starts_with("plan.step.failed") { "LYSO" }
    else if code.starts_with("plan") { "PLAN" }
    else if code.starts_with("benchmark") { "MEMB" }
    else if code.starts_with("peer") { "GOLG" }
    else { "CELL" }
}
