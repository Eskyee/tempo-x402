use crate::api;
use crate::WalletState;
use gloo_timers::callback::Interval;
use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::wallet_panel::WalletButtons;

#[derive(Clone, Debug)]
struct SoulEventMsg { code: String, message: String }

/// The Engine Bay — shows a living intelligence getting smarter.
/// Three zones: INTELLIGENCE (header) → LEARNING LOOP (center) → COLONY (bottom).
#[component]
pub fn Mandala() -> impl IntoView {
    let (wallet, set_wallet) = expect_context::<(ReadSignal<WalletState>, WriteSignal<WalletState>)>();
    let (soul, set_soul) = create_signal(None::<serde_json::Value>);
    let (info, set_info) = create_signal(None::<serde_json::Value>);
    let (system, set_system) = create_signal(None::<serde_json::Value>);
    let (panel_open, set_panel_open) = create_signal(false);
    let (clone_loading, set_clone_loading) = create_signal(false);
    let (clone_result, set_clone_result) = create_signal(None::<Result<String, String>>);
    let (events, set_events) = create_signal(Vec::<SoulEventMsg>::new());
    let (pulses, set_pulses) = create_signal(std::collections::HashMap::<String, f64>::new());

    // Fetch
    let fetch_all = move || {
        spawn_local(async move {
            let base = api::gateway_base_url();
            if let Ok(r) = gloo_net::http::Request::get(&format!("{}/instance/info", base)).send().await {
                if r.ok() { if let Ok(d) = r.json::<serde_json::Value>().await { set_info.set(Some(d)); } }
            }
            if let Ok(d) = api::fetch_soul_status().await { set_soul.set(Some(d)); }
            if let Ok(r) = gloo_net::http::Request::get(&format!("{}/soul/system", base)).send().await {
                if r.ok() { if let Ok(d) = r.json::<serde_json::Value>().await { set_system.set(Some(d)); } }
            }
        });
    };
    fetch_all();
    let interval = Interval::new(8_000, move || { fetch_all(); });
    on_cleanup(move || drop(interval));

    // SSE for real-time events
    {
        let base = api::gateway_base_url().to_string();
        spawn_local(async move {
            let es = match web_sys::EventSource::new(&format!("{}/soul/events/stream", base)) { Ok(e) => e, Err(_) => return };
            let on_msg = Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
                let s = ev.data().as_string().unwrap_or_default();
                if let Ok(p) = serde_json::from_str::<serde_json::Value>(&s) {
                    let code = p.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let msg = p.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if !code.is_empty() && code != "heartbeat" {
                        set_events.update(|e| { e.push(SoulEventMsg { code: code.clone(), message: msg }); if e.len() > 30 { e.drain(..e.len()-30); } });
                        set_pulses.update(|p| { p.insert(code, js_sys::Date::now()); });
                    }
                }
            });
            es.add_event_listener_with_callback("soul_event", on_msg.as_ref().unchecked_ref()).ok();
            on_msg.forget();
        });
    }

    view! {
        <div class="engine-bay">
            <Show when=move || soul.get().is_some() fallback=move || view! {
                <div class="eb-loading">"connecting..."</div>
            }>

            // ═══════════════════════════════════════════════════════════
            // ZONE 1: INTELLIGENCE — the headline
            // ═══════════════════════════════════════════════════════════
            {move || {
                let s = soul.get().unwrap_or_default();
                let b = s.get("benchmark");

                // IQ + delta
                let iq = b.and_then(|b| b.get("opus_iq")).and_then(|v| v.as_str()).unwrap_or("--").to_string();
                let elo = b.and_then(|b| b.get("elo_rating")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                // Solved progress
                let collective = b.and_then(|b| b.get("collective"));
                let unique_solved = collective.and_then(|c| c.get("unique_solved")).and_then(|v| v.as_u64()).unwrap_or(0);
                let total_problems = collective.and_then(|c| c.get("total_problems")).and_then(|v| v.as_u64()).unwrap_or(100);
                let solved_pct = if total_problems > 0 { unique_solved * 100 / total_problems } else { 0 };

                // ELO history sparkline
                let history = b.and_then(|b| b.get("elo_history")).and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let elo_points: Vec<f64> = history.iter()
                    .filter_map(|h| h.get("rating").and_then(|v| v.as_f64()))
                    .collect();
                let elo_min = elo_points.iter().copied().fold(f64::MAX, f64::min);
                let elo_max = elo_points.iter().copied().fold(f64::MIN, f64::max);
                let elo_range = (elo_max - elo_min).max(1.0);

                // Alpha / Psi / Free Energy
                let accel = s.get("acceleration");
                let alpha: f64 = accel.and_then(|a| a.get("alpha")).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
                    .or_else(|| accel.and_then(|a| a.get("alpha")).and_then(|v| v.as_f64()))
                    .unwrap_or(0.0);
                let regime = accel.and_then(|a| a.get("regime")).and_then(|v| v.as_str()).unwrap_or("STALLED");
                let alpha_sym = match regime { "ACCELERATING" => "\u{25B2}", "DECELERATING" => "\u{25BC}", "CRUISING" => "\u{25C6}", _ => "\u{25CB}" };
                let alpha_cls = match regime { "ACCELERATING" => "acc-up", "DECELERATING" => "acc-down", "CRUISING" => "acc-flat", _ => "acc-stall" };

                let role = s.get("role");
                let psi = role.and_then(|r| r.get("psi")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                let fe = s.get("free_energy");
                let f_val = fe.and_then(|f| f.get("F")).and_then(|v| v.as_str()).unwrap_or("--").to_string();
                let fe_regime = fe.and_then(|f| f.get("regime")).and_then(|v| v.as_str()).unwrap_or("--").to_string();

                view! {
                    <div class="eb-zone-intel">
                        // Row 1: IQ (hero) + solved + ELO
                        <div class="eb-intel-row">
                            <span class="eb-iq-hero">{format!("IQ {}", iq)}</span>
                            <span class="eb-solved-hero">{format!("{}/{} SOLVED", unique_solved, total_problems)}</span>
                            <span class="eb-elo-hero">{format!("ELO {:.0}", elo)}</span>
                        </div>

                        // Row 2: Progress bar (the ONE bar that matters)
                        <div class="eb-progress-wrap">
                            <div class="eb-progress-bar">
                                <div class="eb-progress-fill" style=format!("width:{}%", solved_pct)></div>
                            </div>
                            // Mini sparkline of ELO history
                            {(!elo_points.is_empty()).then(|| view! {
                                <svg class="eb-sparkline" viewBox="0 0 60 16">
                                    {elo_points.iter().enumerate().map(|(i, &val)| {
                                        let x = if elo_points.len() > 1 { (i as f64 / (elo_points.len() - 1) as f64) * 56.0 + 2.0 } else { 30.0 };
                                        let y = 14.0 - ((val - elo_min) / elo_range) * 12.0;
                                        view! { <circle cx=x.to_string() cy=y.to_string() r="1.5" fill="#00ff41" opacity="0.7"/> }
                                    }).collect::<Vec<_>>()}
                                    // Connect with lines
                                    {elo_points.windows(2).enumerate().map(|(i, w)| {
                                        let x1 = if elo_points.len() > 1 { (i as f64 / (elo_points.len() - 1) as f64) * 56.0 + 2.0 } else { 30.0 };
                                        let x2 = if elo_points.len() > 1 { ((i+1) as f64 / (elo_points.len() - 1) as f64) * 56.0 + 2.0 } else { 30.0 };
                                        let y1 = 14.0 - ((w[0] - elo_min) / elo_range) * 12.0;
                                        let y2 = 14.0 - ((w[1] - elo_min) / elo_range) * 12.0;
                                        view! { <line x1=x1.to_string() y1=y1.to_string() x2=x2.to_string() y2=y2.to_string() stroke="#00ff41" stroke-width="0.5" opacity="0.4"/> }
                                    }).collect::<Vec<_>>()}
                                </svg>
                            })}
                        </div>

                        // Row 3: Subdued metrics
                        <div class="eb-intel-sub">
                            <span class={format!("eb-alpha {}", alpha_cls)}>{format!("\u{03B1} {} {}", alpha_sym, regime)}</span>
                            <span class="eb-psi-sub">{format!("\u{03A8}={:.3}", psi)}</span>
                            <span class="eb-fe-sub">
                                {format!("F={}", f_val)}
                                <span class={format!("eb-regime-tag {}", fe_regime.to_lowercase())}>{fe_regime}</span>
                            </span>
                        </div>
                    </div>
                }
            }}

            // ═══════════════════════════════════════════════════════════
            // ZONE 2: THE LEARNING LOOP
            // BENCHMARK → TRAIN → GENERATE → VERIFY
            // ═══════════════════════════════════════════════════════════
            <div class="eb-loop">
                // ── BENCHMARK node ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let b = s.get("benchmark");
                    let pass = b.and_then(|b| b.get("pass_at_1")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let opus = b.and_then(|b| b.get("opus"));
                    let session_passed = opus.and_then(|o| o.get("problems_passed")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let session_attempted = opus.and_then(|o| o.get("problems_attempted")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let history = b.and_then(|b| b.get("elo_history")).and_then(|v| v.as_array());
                    let runs = history.map(|h| h.len()).unwrap_or(0);
                    let p = pulses.get();
                    let glow = if pulse_active(&p, "benchmark") { " eb-node-glow" } else { "" };
                    view! {
                        <div class={format!("eb-loop-node eb-node-benchmark{}", glow)}>
                            <div class="eb-node-label">"BENCHMARK"</div>
                            <div class="eb-node-stat eb-node-big">{format!("{:.1}% pass@1", pass)}</div>
                            <div class="eb-node-stat">{format!("{}/{} session", session_passed, session_attempted)}</div>
                            <div class="eb-node-stat eb-dim">{format!("{} runs total", runs)}</div>
                        </div>
                    }
                }}
                <div class="eb-loop-arrow">{"\u{2192}"}</div>

                // ── TRAIN node ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let brain_loss = s.get("brain").and_then(|b| b.get("running_loss")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let brain_steps = s.get("brain").and_then(|b| b.get("train_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let cg_loss = s.get("codegen").and_then(|c| c.get("model_loss")).and_then(|v| v.as_f64())
                        .or_else(|| s.get("codegen").and_then(|c| c.get("model_loss")).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()))
                        .unwrap_or(0.0);
                    let cg_steps = s.get("codegen").and_then(|c| c.get("model_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let xf_loss = s.get("transformer").and_then(|t| t.get("last_train_loss")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                    let p = pulses.get();
                    let glow = if pulse_active(&p, "brain") || pulse_active(&p, "transformer") || pulse_active(&p, "codegen") { " eb-node-glow" } else { "" };

                    view! {
                        <div class={format!("eb-loop-node eb-node-train{}", glow)}>
                            <div class="eb-node-label">"TRAIN"</div>
                            {render_loss_line("brain", brain_loss, brain_steps)}
                            {render_loss_line("codegen", cg_loss, cg_steps)}
                            {render_loss_line("xformer", xf_loss, 0)}
                        </div>
                    }
                }}
                <div class="eb-loop-arrow">{"\u{2192}"}</div>

                // ── GENERATE node (codegen weaning) ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let cg = s.get("codegen");
                    let solutions = cg.and_then(|c| c.get("solutions_stored")).and_then(|v| v.as_u64()).unwrap_or(0);
                    let can = cg.and_then(|c| c.get("can_generate")).and_then(|v| v.as_bool()).unwrap_or(false);
                    let loss = cg.and_then(|c| c.get("model_loss")).and_then(|v| v.as_f64())
                        .or_else(|| cg.and_then(|c| c.get("model_loss")).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()))
                        .unwrap_or(10.0);
                    let params = cg.and_then(|c| c.get("model_params")).and_then(|v| v.as_u64()).unwrap_or(0);

                    let status = if !can { "bootstrapping..." }
                        else if loss > 4.0 { "learning patterns..." }
                        else if loss > 2.0 { "generating tokens..." }
                        else { "generating Rust!" };
                    let status_cls = if loss < 2.0 { "eb-gen-hot" } else if loss < 4.0 { "eb-gen-warm" } else { "eb-gen-cold" };

                    view! {
                        <div class="eb-loop-node eb-node-generate">
                            <div class="eb-node-label">"GENERATE"</div>
                            <div class={format!("eb-node-stat {}", status_cls)}>{status}</div>
                            <div class="eb-node-stat">{format!("{}M params", params / 1_000_000)}</div>
                            <div class="eb-node-stat">{format!("{} solutions stored", solutions)}</div>
                        </div>
                    }
                }}
                <div class="eb-loop-arrow">{"\u{2192}"}</div>

                // ── VERIFY node (the 9 cognitive systems as dots) ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let systems = [
                        ("B", "brain", s.get("brain").and_then(|b| b.get("running_loss")).and_then(|v| v.as_f64()).unwrap_or(1.0)),
                        ("C", "cortex", {
                            let acc_str = s.get("cortex").and_then(|c| c.get("prediction_accuracy")).and_then(|v| v.as_str()).unwrap_or("0");
                            let acc: f64 = acc_str.replace('%', "").parse().unwrap_or(0.0);
                            1.0 - (acc / 100.0)
                        }),
                        ("G", "genesis", 0.2), // always healthy if running
                        ("H", "hivemind", {
                            let trails = s.get("hivemind").and_then(|h| h.get("total_trails")).and_then(|v| v.as_u64()).unwrap_or(0);
                            if trails > 10 { 0.2 } else { 0.6 }
                        }),
                        ("S", "synthesis", {
                            let state = s.get("synthesis").and_then(|sy| sy.get("state")).and_then(|v| v.as_str()).unwrap_or("stuck");
                            match state { "coherent" | "exploiting" => 0.1, "exploring" => 0.3, "conflicted" => 0.6, _ => 0.9 }
                        }),
                        ("E", "eval", 0.2),
                        ("A", "autonomy", 0.3),
                        ("F", "feedback", {
                            let ch = s.get("cycle_health");
                            let comp = ch.and_then(|h| h.get("completed_plans_count")).and_then(|v| v.as_u64()).unwrap_or(0);
                            let fail = ch.and_then(|h| h.get("failed_plans_count")).and_then(|v| v.as_u64()).unwrap_or(0);
                            if comp + fail == 0 { 0.5 } else { fail as f64 / (comp + fail) as f64 }
                        }),
                        ("\u{03A8}", "free_energy", {
                            let f_str = s.get("free_energy").and_then(|f| f.get("F")).and_then(|v| v.as_str()).unwrap_or("0.5");
                            f_str.parse::<f64>().unwrap_or(0.5)
                        }),
                    ];

                    let p = pulses.get();

                    view! {
                        <div class="eb-loop-node eb-node-verify">
                            <div class="eb-node-label">"VERIFY"</div>
                            <div class="eb-systems-row">
                                {systems.iter().map(|(label, prefix, badness)| {
                                    let color = if *badness < 0.3 { "var(--green)" } else if *badness < 0.6 { "var(--amber)" } else { "var(--red)" };
                                    let glow = if pulse_active(&p, prefix) { "eb-sys-pulse" } else { "" };
                                    view! {
                                        <div class={format!("eb-sys-dot {}", glow)} style=format!("color:{}", color) title=prefix.to_string()>
                                            {label.to_string()}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                }}
            </div>

            // ═══════════════════════════════════════════════════════════
            // ZONE 3: COLONY + EVENTS
            // ═══════════════════════════════════════════════════════════
            <div class="eb-zone-colony">
                // Colony peers
                {move || {
                    let d = info.get().unwrap_or_default();
                    let peers = d.get("peers").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    let self_fitness = d.get("fitness").and_then(|f| f.get("total")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let s = soul.get().unwrap_or_default();
                    let colony_size = s.get("role").and_then(|r| r.get("colony_size")).and_then(|v| v.as_u64()).unwrap_or(1);
                    view! {
                        <div class="eb-colony-row">
                            <span class="eb-colony-label">{format!("COLONY ({})", colony_size)}</span>
                            <div class="eb-colony-dot eb-colony-self" title="self">
                                <span>{format!("{:.0}%", self_fitness * 100.0)}</span>
                            </div>
                            {peers.iter().take(5).map(|peer| {
                                let fit = peer.get("fitness").and_then(|f| f.get("total")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let id = peer.get("instance_id").and_then(|v| v.as_str()).unwrap_or("?");
                                let short: String = id.chars().take(6).collect();
                                view! {
                                    <div class="eb-colony-dot" title=short.clone()>
                                        <span>{format!("{:.0}%", fit * 100.0)}</span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }}

                // Events stream
                <div class="eb-events">
                    {move || {
                        let evts = events.get();
                        evts.iter().rev().take(5).map(|evt| {
                            let color = event_color(&evt.code);
                            let abbr = event_abbr(&evt.code);
                            let msg: String = evt.message.chars().take(70).collect();
                            view! {
                                <div class="eb-event">
                                    <span class="eb-event-tag" style=format!("color:{}", color)>{format!("[{}]", abbr)}</span>
                                    <span class="eb-event-msg">{msg}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>

            // Status bar
            {move || {
                let s = soul.get().unwrap_or_default();
                let cycles = s.get("total_cycles").and_then(|v| v.as_u64()).unwrap_or(0);
                let mode = s.get("mode").and_then(|v| v.as_str()).unwrap_or("--");
                let d = info.get().unwrap_or_default();
                let fitness = d.get("fitness").and_then(|f| f.get("total")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let active = s.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                let sys = system.get().unwrap_or_default();
                let cpu = sys.get("cpu_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                view! {
                    <div class="eb-bottom">
                        <span class={if active { "eb-dot-on" } else { "eb-dot-off" }}></span>
                        <span>{format!("{} | cycle {} | fitness {:.0}% | cpu {:.0}%", mode, cycles, fitness * 100.0, cpu)}</span>
                        <span style="flex:1"></span>
                        <span class="eb-ver">{concat!("v", env!("CARGO_PKG_VERSION"))}</span>
                    </div>
                }
            }}

            </Show>

            // Controls
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
                            if !avail { return view! { <div></div> }.into_view(); }
                            let price = d.get("clone_price").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();
                            let do_clone = move |_: web_sys::MouseEvent| {
                                if clone_loading.get() { return; }
                                let w = wallet.get();
                                if !w.connected { return; }
                                set_clone_loading.set(true); set_clone_result.set(None);
                                spawn_local(async move {
                                    match api::clone_instance(&w).await {
                                        Ok(r) => set_clone_result.set(Some(Ok(format!("{} at {}", r.instance_id.unwrap_or_default(), r.url.unwrap_or_default())))),
                                        Err(e) => set_clone_result.set(Some(Err(e))),
                                    }
                                    set_clone_loading.set(false);
                                });
                            };
                            view! {
                                <div class="mandala-panel-section">
                                    <div class="mandala-panel-label">"CLONE"</div>
                                    <button class="btn btn-primary" on:click=do_clone disabled=move || clone_loading.get() || !wallet.get().connected>
                                        {move || if clone_loading.get() { "..." } else { "Clone" }}
                                    </button>
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

// ── Helpers ──

fn render_loss_line(name: &str, loss: f64, steps: u64) -> impl IntoView {
    let color = if loss < 0.5 { "var(--green)" } else if loss < 2.0 { "var(--amber)" } else { "var(--red)" };
    let steps_str = if steps > 0 { format!(" {}K", steps / 1000) } else { String::new() };
    view! {
        <div class="eb-loss-line">
            <span class="eb-loss-name">{name.to_string()}</span>
            <span class="eb-loss-val" style=format!("color:{}", color)>{format!("{:.2}", loss)}</span>
            <span class="eb-loss-steps">{steps_str}</span>
        </div>
    }
}

fn pulse_active(pulses: &std::collections::HashMap<String, f64>, prefix: &str) -> bool {
    let now = js_sys::Date::now();
    pulses.iter().any(|(c, t)| c.starts_with(prefix) && (now - t) < 8_000.0)
}

fn event_color(code: &str) -> &'static str {
    if code.starts_with("brain") { "#00ff41" }
    else if code.starts_with("transformer") { "#00e5ff" }
    else if code.starts_with("codegen") { "#ffa000" }
    else if code.starts_with("plan") { "#b388ff" }
    else if code.starts_with("benchmark") { "#00ff41" }
    else if code.starts_with("peer") { "#00e5ff" }
    else { "#5a6a5a" }
}

fn event_abbr(code: &str) -> &'static str {
    if code.starts_with("brain.trained") { "BRAIN" }
    else if code.starts_with("transformer") { "XFORM" }
    else if code.starts_with("codegen") { "COGEN" }
    else if code.starts_with("plan.step") { "STEP" }
    else if code.starts_with("plan") { "PLAN" }
    else if code.starts_with("benchmark") { "BENCH" }
    else if code.starts_with("peer") { "PEER" }
    else { "EVENT" }
}
