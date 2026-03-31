use crate::api;
use crate::WalletState;
use gloo_timers::callback::Interval;
use leptos::*;

use super::wallet_panel::WalletButtons;

/// Neural Mandala — alien intelligence visualization.
/// Full viewport SVG. Ψ orb at center, cognitive systems orbiting,
/// connections pulsing, sparkline rings, colony at the edges.
#[component]
pub fn Mandala() -> impl IntoView {
    let (wallet, set_wallet) =
        expect_context::<(ReadSignal<WalletState>, WriteSignal<WalletState>)>();

    let (soul, set_soul) = create_signal(None::<serde_json::Value>);
    let (info, set_info) = create_signal(None::<serde_json::Value>);
    let (system, set_system) = create_signal(None::<serde_json::Value>);
    let (tick, set_tick) = create_signal(0u32);
    let (panel_open, set_panel_open) = create_signal(false);
    let (clone_loading, set_clone_loading) = create_signal(false);
    let (clone_result, set_clone_result) = create_signal(None::<Result<String, String>>);

    // History buffers for sparkline rings
    let (psi_history, set_psi_history) = create_signal(Vec::<f64>::new());
    let (fe_history, set_fe_history) = create_signal(Vec::<f64>::new());
    let (fitness_history, set_fitness_history) = create_signal(Vec::<f64>::new());

    let fetch_all = move || {
        spawn_local(async move {
            let base = api::gateway_base_url();
            if let Ok(resp) = gloo_net::http::Request::get(&format!("{}/instance/info", base))
                .send().await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        set_info.set(Some(data));
                    }
                }
            }
            if let Ok(data) = api::fetch_soul_status().await {
                // Track history
                let psi = data.get("role").and_then(|r| r.get("psi")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let fe = data.get("free_energy").and_then(|f| f.get("F")).and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let fit = data.get("fitness").and_then(|f| f.get("total")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                set_psi_history.update(|h| { h.push(psi); if h.len() > 60 { h.drain(..h.len()-60); } });
                set_fe_history.update(|h| { h.push(fe); if h.len() > 60 { h.drain(..h.len()-60); } });
                set_fitness_history.update(|h| { h.push(fit); if h.len() > 60 { h.drain(..h.len()-60); } });

                set_soul.set(Some(data));
            }
            if let Ok(resp) = gloo_net::http::Request::get(&format!("{}/soul/system", base))
                .send().await
            {
                if resp.ok() {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        set_system.set(Some(data));
                    }
                }
            }
        });
    };

    fetch_all();
    let interval = Interval::new(10_000, move || {
        set_tick.update(|t| *t = t.wrapping_add(1));
        fetch_all();
    });
    on_cleanup(move || drop(interval));

    // Layout constants — square viewBox, works in portrait + landscape
    let cx = 400.0f64;
    let cy = 400.0f64;
    let r_inner = 100.0f64;
    let r_outer = 185.0f64;
    let r_colony = 280.0f64;
    let r_spark_psi = 225.0f64;
    let r_spark_fe = 238.0f64;
    let r_spark_fit = 251.0f64;

    // 9 cognitive systems
    let systems = [
        ("BRAIN", 0), ("CORTEX", 1), ("GENESIS", 2), ("HIVEMND", 3),
        ("SYNTH", 4), ("EVAL", 5), ("AUTON", 6), ("FREE-E", 7), ("FEEDBACK", 8),
    ];

    // 4 models
    let models = [
        ("brain", 0), ("xformer", 1), ("quality", 2), ("codegen", 3),
    ];

    view! {
        <div class="mandala-container">
            <svg viewBox="0 0 800 800" class="mandala-svg" preserveAspectRatio="xMidYMid meet">
                <defs>
                    // Glow filters
                    <filter id="glow-green">
                        <feGaussianBlur stdDeviation="4" result="blur"/>
                        <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
                    </filter>
                    <filter id="glow-cyan">
                        <feGaussianBlur stdDeviation="3" result="blur"/>
                        <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
                    </filter>
                    <filter id="glow-psi">
                        <feGaussianBlur stdDeviation="8" result="blur"/>
                        <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
                    </filter>
                    <radialGradient id="psi-grad" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stop-color="#00ff41" stop-opacity="0.6"/>
                        <stop offset="100%" stop-color="#00ff41" stop-opacity="0"/>
                    </radialGradient>
                    <radialGradient id="orb-inner" cx="40%" cy="35%" r="60%">
                        <stop offset="0%" stop-color="#00ff88"/>
                        <stop offset="100%" stop-color="#005522"/>
                    </radialGradient>
                </defs>

                // ── Background grid (subtle) ──
                {(0..10).map(|i| {
                    let r = (i as f64 + 1.0) * 40.0;
                    view! {
                        <circle cx=cx.to_string() cy=cy.to_string() r=r.to_string()
                            fill="none" stroke="#0a0a1a" stroke-width="0.5"/>
                    }
                }).collect::<Vec<_>>()}

                // ── Connections: models ↔ center ──
                {move || {
                    let t = tick.get();
                    models.iter().map(|(_, i)| {
                        let angle = (*i as f64) * std::f64::consts::TAU / 4.0 - std::f64::consts::FRAC_PI_2;
                        let mx = cx + r_inner * angle.cos();
                        let my = cy + r_inner * angle.sin();
                        // Animate dash offset
                        let offset = (t as f64 * 2.0 + *i as f64 * 10.0) % 20.0;
                        view! {
                            <line x1=cx.to_string() y1=cy.to_string()
                                x2=mx.to_string() y2=my.to_string()
                                stroke="#00ff41" stroke-width="0.8" stroke-opacity="0.15"
                                stroke-dasharray="4 4"
                                stroke-dashoffset=offset.to_string()
                            />
                        }
                    }).collect::<Vec<_>>()
                }}

                // ── Connections: systems ↔ models ──
                {move || {
                    let t = tick.get();
                    systems.iter().map(|(_, i)| {
                        let angle = (*i as f64) * std::f64::consts::TAU / 9.0 - std::f64::consts::FRAC_PI_2;
                        let sx = cx + r_outer * angle.cos();
                        let sy = cy + r_outer * angle.sin();
                        // Connect to nearest model
                        let mi = *i % 4;
                        let m_angle = (mi as f64) * std::f64::consts::TAU / 4.0 - std::f64::consts::FRAC_PI_2;
                        let mx = cx + r_inner * m_angle.cos();
                        let my = cy + r_inner * m_angle.sin();
                        let offset = (t as f64 * 1.5 + *i as f64 * 7.0) % 16.0;
                        view! {
                            <line x1=sx.to_string() y1=sy.to_string()
                                x2=mx.to_string() y2=my.to_string()
                                stroke="#00e5ff" stroke-width="0.5" stroke-opacity="0.1"
                                stroke-dasharray="3 5"
                                stroke-dashoffset=offset.to_string()
                            />
                        }
                    }).collect::<Vec<_>>()
                }}

                // ── Sparkline rings ──
                {move || render_sparkline_ring(&psi_history.get(), cx, cy, r_spark_psi, "#00ff41", 0.4)}
                {move || render_sparkline_ring(&fe_history.get(), cx, cy, r_spark_fe, "#00e5ff", 0.3)}
                {move || render_sparkline_ring(&fitness_history.get(), cx, cy, r_spark_fit, "#ffa000", 0.3)}

                // ── Ψ center orb ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let role = s.get("role");
                    let psi = role.and_then(|r| r.get("psi")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let fe = s.get("free_energy");
                    let regime = fe.and_then(|f| f.get("regime")).and_then(|v| v.as_str()).unwrap_or("EXPLOIT");
                    let f_val = fe.and_then(|f| f.get("F")).and_then(|v| v.as_str()).unwrap_or("--");

                    let orb_r = 25.0 + psi * 30.0; // 25-55px based on Ψ
                    let (orb_color, glow_color) = match regime {
                        "EXPLORE" => ("#00e5ff", "#003344"),
                        "LEARN" => ("#b388ff", "#2a1a44"),
                        "EXPLOIT" => ("#00ff41", "#003311"),
                        "ANOMALY" => ("#ff1744", "#440011"),
                        _ => ("#00ff41", "#003311"),
                    };

                    let iq = s.get("benchmark").and_then(|b| b.get("opus_iq")).and_then(|v| v.as_str()).unwrap_or("--");

                    view! {
                        // Outer glow
                        <circle cx=cx.to_string() cy=cy.to_string() r=(orb_r * 2.5).to_string()
                            fill="url(#psi-grad)" opacity="0.3" filter="url(#glow-psi)"/>
                        // Orb
                        <circle cx=cx.to_string() cy=cy.to_string() r=orb_r.to_string()
                            fill=glow_color stroke=orb_color stroke-width="2"
                            filter="url(#glow-green)" class="psi-orb"/>
                        // Inner bright spot
                        <circle cx=(cx - orb_r * 0.2).to_string() cy=(cy - orb_r * 0.2).to_string()
                            r=(orb_r * 0.3).to_string()
                            fill=orb_color opacity="0.3"/>
                        // Ψ label
                        <text x=cx.to_string() y=(cy - 2.0).to_string()
                            text-anchor="middle" class="mandala-text-psi" fill=orb_color>
                            {format!("\u{03A8} {:.3}", psi)}
                        </text>
                        // F below
                        <text x=cx.to_string() y=(cy + 12.0).to_string()
                            text-anchor="middle" class="mandala-text-small" fill="#ffffff" opacity="0.5">
                            {format!("F={} {}", f_val, regime)}
                        </text>
                        // IQ above orb
                        <text x=cx.to_string() y=(cy - orb_r - 12.0).to_string()
                            text-anchor="middle" class="mandala-text-iq" fill=orb_color>
                            {format!("IQ {}", iq)}
                        </text>
                    }
                }}

                // ── Model ring (inner) ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    models.iter().map(|(name, i)| {
                        let angle = (*i as f64) * std::f64::consts::TAU / 4.0 - std::f64::consts::FRAC_PI_2;
                        let mx = cx + r_inner * angle.cos();
                        let my = cy + r_inner * angle.sin();

                        let (node_r, color, label) = match *name {
                            "brain" => {
                                let b = s.get("brain");
                                let loss = b.and_then(|b| b.get("running_loss")).and_then(|v| v.as_f64()).unwrap_or(1.0);
                                let steps = b.and_then(|b| b.get("train_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                                let brightness = (1.0 - loss.min(1.0)) * 0.8 + 0.2;
                                (8.0 + (steps as f64 / 5000.0).min(8.0), format!("rgba(0,255,65,{:.2})", brightness), format!("{}K", steps/1000))
                            }
                            "xformer" => {
                                let t = s.get("transformer");
                                let loss = t.and_then(|t| t.get("last_train_loss")).and_then(|v| v.as_f64()).unwrap_or(2.0);
                                let steps = t.and_then(|t| t.get("train_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                                let brightness = (1.0 - (loss / 2.0).min(1.0)) * 0.8 + 0.2;
                                (8.0 + (steps as f64 / 1000.0).min(8.0), format!("rgba(0,229,255,{:.2})", brightness), format!("{}K", steps/1000))
                            }
                            "quality" => {
                                let q = s.get("quality");
                                let steps = q.and_then(|q| q.get("train_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                                (6.0 + (steps as f64 / 500.0).min(6.0), "rgba(179,136,255,0.6)".to_string(), format!("{}s", steps))
                            }
                            "codegen" => {
                                let cg = s.get("codegen");
                                let steps = cg.and_then(|c| c.get("model_steps")).and_then(|v| v.as_u64()).unwrap_or(0);
                                let sols = cg.and_then(|c| c.get("solutions_stored")).and_then(|v| v.as_u64()).unwrap_or(0);
                                let can = cg.and_then(|c| c.get("can_generate")).and_then(|v| v.as_bool()).unwrap_or(false);
                                let color = if can { "rgba(0,255,65,0.8)" } else if sols > 0 { "rgba(255,160,0,0.6)" } else { "rgba(255,23,68,0.3)" };
                                (6.0 + (steps as f64 / 50.0).min(10.0), color.to_string(), format!("{}d", sols))
                            }
                            _ => (6.0, "rgba(100,100,100,0.5)".to_string(), String::new()),
                        };

                        view! {
                            <circle cx=mx.to_string() cy=my.to_string() r=node_r.to_string()
                                fill=color.clone() stroke=color stroke-width="1"
                                filter="url(#glow-cyan)"/>
                            <text x=mx.to_string() y=(my + node_r + 10.0).to_string()
                                text-anchor="middle" class="mandala-text-tiny" fill="#5a6a5a">
                                {name.to_string()}
                            </text>
                            <text x=mx.to_string() y=(my + node_r + 19.0).to_string()
                                text-anchor="middle" class="mandala-text-tiny" fill="#3a4a3a">
                                {label}
                            </text>
                        }
                    }).collect::<Vec<_>>()
                }}

                // ── System ring (outer) ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let synth = s.get("synthesis");
                    systems.iter().map(|(name, i)| {
                        let angle = (*i as f64) * std::f64::consts::TAU / 9.0 - std::f64::consts::FRAC_PI_2;
                        let sx = cx + r_outer * angle.cos();
                        let sy = cy + r_outer * angle.sin();

                        // Get health color from system data
                        let (node_r, color) = system_health(&s, name);

                        view! {
                            <circle cx=sx.to_string() cy=sy.to_string() r=node_r.to_string()
                                fill="none" stroke=color.clone() stroke-width="1.5"
                                opacity="0.7"/>
                            // Inner fill (dimmer)
                            <circle cx=sx.to_string() cy=sy.to_string() r=(node_r * 0.6).to_string()
                                fill=color.clone() opacity="0.15"/>
                            <text x=sx.to_string() y=(sy + 1.0).to_string()
                                text-anchor="middle" class="mandala-text-system" fill=color>
                                {name.to_string()}
                            </text>
                        }
                    }).collect::<Vec<_>>()
                }}

                // ── Colony peers (far ring) ──
                {move || {
                    let d = info.get().unwrap_or_default();
                    let peers = d.get("peers")
                        .or_else(|| d.get("children"))
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    if peers.is_empty() { return vec![]; }
                    let n = peers.len();
                    peers.iter().enumerate().map(|(i, p)| {
                        let angle = (i as f64) * std::f64::consts::TAU / (n as f64) - std::f64::consts::FRAC_PI_2;
                        let px = cx + r_colony * angle.cos();
                        let py = cy + r_colony * angle.sin();
                        let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        let color = if status == "running" { "#00ff41" } else { "#ff1744" };
                        let id = p.get("instance_id").and_then(|v| v.as_str()).unwrap_or("?");
                        let short = if id.len() > 6 { &id[..6] } else { id };
                        view! {
                            // Sync line to center
                            <line x1=cx.to_string() y1=cy.to_string()
                                x2=px.to_string() y2=py.to_string()
                                stroke=color stroke-width="0.3" stroke-opacity="0.15"
                                stroke-dasharray="2 6"/>
                            <circle cx=px.to_string() cy=py.to_string() r="4"
                                fill="none" stroke=color stroke-width="1" opacity="0.5"/>
                            <text x=px.to_string() y=(py + 12.0).to_string()
                                text-anchor="middle" class="mandala-text-tiny" fill="#3a4a3a">
                                {short.to_string()}
                            </text>
                        }
                    }).collect::<Vec<_>>()
                }}

                // ── Fitness + key metrics overlay ──
                {move || {
                    let d = info.get().unwrap_or_default();
                    let s = soul.get().unwrap_or_default();
                    let fitness = d.get("fitness").and_then(|f| f.get("total")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let cycles = s.get("total_cycles").and_then(|v| v.as_u64()).unwrap_or(0);
                    let mode = s.get("mode").and_then(|v| v.as_str()).unwrap_or("--");
                    let active = s.get("active").and_then(|v| v.as_bool()).unwrap_or(false);

                    let bench = s.get("benchmark");
                    let pass = bench.and_then(|b| b.get("pass_at_1")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let elo = bench.and_then(|b| b.get("elo_rating")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                    let sys = system.get().unwrap_or_default();
                    let cpu = sys.get("cpu_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let mem = sys.get("mem_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);

                    let status_color = if active { "#00ff41" } else { "#ff1744" };

                    view! {
                        // Top left: identity
                        <text x="16" y="20" class="mandala-text-label" fill="#3a4a3a">
                            {format!("tempo-x402 | {} | cycle {} | cpu {:.0}% mem {:.0}%", mode, cycles, cpu, mem)}
                        </text>
                        // Top right: nav
                        <a href="/studio">
                            <text x="784" y="20" text-anchor="end" class="mandala-text-label" fill="#5a6a5a">
                                "STUDIO \u{2192}"
                            </text>
                        </a>
                        // Bottom left: fitness
                        <text x="16" y="780" class="mandala-text-label" fill="#ffa000">
                            {format!("fitness {:.0}%", fitness * 100.0)}
                        </text>
                        // Bottom center: pass@1 + ELO
                        <text x="400" y="780" text-anchor="middle" class="mandala-text-label" fill="#5a6a5a">
                            {format!("pass@1 {:.1}% | ELO {:.0}", pass, elo)}
                        </text>
                        // Bottom right: version
                        <text x="784" y="780" text-anchor="end" class="mandala-text-label" fill="#2a2a3a">
                            {concat!("v", env!("CARGO_PKG_VERSION"))}
                        </text>
                        // Status dot
                        <circle cx="10" cy="16" r="3" fill=status_color/>
                    }
                }}

                // ── Recent thoughts as flowing particles ──
                {move || {
                    let s = soul.get().unwrap_or_default();
                    let thoughts: Vec<serde_json::Value> = s.get("recent_thoughts")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let t = tick.get();
                    thoughts.iter().rev().take(12).enumerate().map(|(i, thought)| {
                        let tt = thought.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                        let color = match tt {
                            "observation" => "#00e5ff",
                            "reasoning" => "#00ff41",
                            "decision" => "#ffa000",
                            "reflection" => "#b388ff",
                            "tool_execution" => "#ffa000",
                            "mutation" => "#ff1744",
                            "memory_consolidation" => "#5a6a5a",
                            _ => "#3a4a3a",
                        };
                        // Particles orbit at different radii based on recency
                        let orbit_r = 160.0 + (i as f64) * 12.0;
                        let speed = 0.02 - (i as f64) * 0.001;
                        let phase = (t as f64) * speed + (i as f64) * 0.7;
                        let px = cx + orbit_r * phase.cos();
                        let py = cy + orbit_r * phase.sin();
                        let size = 2.5 - (i as f64) * 0.15;
                        let opacity = 0.8 - (i as f64) * 0.05;
                        view! {
                            <circle cx=px.to_string() cy=py.to_string() r=size.to_string()
                                fill=color opacity=opacity.to_string()/>
                        }
                    }).collect::<Vec<_>>()
                }}
            </svg>

            // ── Floating control panel (top-right) ──
            <div class="mandala-controls">
                <button class="mandala-toggle" on:click=move |_| set_panel_open.update(|v| *v = !*v)>
                    {move || if panel_open.get() { "\u{2715}" } else { "\u{2630}" }}
                </button>

                <Show when=move || panel_open.get() fallback=|| ()>
                    <div class="mandala-panel">
                        // Wallet
                        <div class="mandala-panel-section">
                            <div class="mandala-panel-label">"ACCOUNT"</div>
                            <WalletButtons wallet=wallet set_wallet=set_wallet />
                        </div>

                        // Balance + address
                        {move || {
                            let w = wallet.get();
                            if !w.connected { return view! { <div></div> }.into_view(); }
                            let addr = w.address.unwrap_or_default();
                            let short = if addr.len() > 10 { format!("{}...{}", &addr[..6], &addr[addr.len()-4..]) } else { addr };
                            view! {
                                <div class="mandala-panel-section">
                                    <div style="font-size:10px;color:var(--text-dim)">{short}</div>
                                </div>
                            }.into_view()
                        }}

                        // Clone button
                        {move || {
                            let d = info.get().unwrap_or_default();
                            let clone_available = d.get("clone_available").and_then(|v| v.as_bool()).unwrap_or(false);
                            let clone_price = d.get("clone_price").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();
                            if !clone_available { return view! { <div></div> }.into_view(); }

                            let do_clone = move |_: web_sys::MouseEvent| {
                                if clone_loading.get() { return; }
                                let w = wallet.get();
                                if !w.connected { return; }
                                set_clone_loading.set(true);
                                set_clone_result.set(None);
                                spawn_local(async move {
                                    match api::clone_instance(&w).await {
                                        Ok(resp) => {
                                            let msg = format!("Clone {} at {}", resp.instance_id.unwrap_or_default(), resp.url.unwrap_or_default());
                                            set_clone_result.set(Some(Ok(msg)));
                                        }
                                        Err(e) => set_clone_result.set(Some(Err(e))),
                                    }
                                    set_clone_loading.set(false);
                                });
                            };

                            view! {
                                <div class="mandala-panel-section">
                                    <div class="mandala-panel-label">"CLONE"</div>
                                    <button class="btn btn-primary"
                                        on:click=do_clone
                                        disabled=move || clone_loading.get() || !wallet.get().connected
                                    >
                                        {move || if clone_loading.get() { "Cloning..." } else { "Clone Node" }}
                                    </button>
                                    <div style="font-size:9px;color:var(--text-muted);margin-top:2px">
                                        {format!("${}", clone_price)}
                                    </div>
                                    {move || clone_result.get().map(|r| match r {
                                        Ok(msg) => view! { <div style="font-size:9px;color:var(--green);margin-top:4px">{msg}</div> }.into_view(),
                                        Err(e) => view! { <div style="font-size:9px;color:var(--red);margin-top:4px">{e}</div> }.into_view(),
                                    })}
                                </div>
                            }.into_view()
                        }}

                        // Navigation
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

/// Render a sparkline as an arc ring around the center.
fn render_sparkline_ring(
    data: &[f64],
    cx: f64,
    cy: f64,
    radius: f64,
    color: &str,
    opacity: f64,
) -> impl IntoView {
    if data.len() < 2 {
        return view! { <g></g> }.into_view();
    }
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.001);

    let n = data.len();
    let arc_span = std::f64::consts::TAU * 0.75;
    let start_angle = std::f64::consts::FRAC_PI_2 + std::f64::consts::FRAC_PI_4;

    let points: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let t = i as f64 / (n - 1) as f64;
            let angle = start_angle + t * arc_span;
            let norm = (v - min) / range;
            let r = radius + norm * 15.0 - 7.5;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            format!("{:.1},{:.1}", x, y)
        })
        .collect();

    let path_d = format!("M {} L {}", points[0], points[1..].join(" L "));
    let color = color.to_string();
    let opacity = opacity.to_string();

    view! {
        <path d=path_d fill="none" stroke=color stroke-width="1"
            opacity=opacity stroke-linecap="round"/>
    }
    .into_view()
}

/// Get visual size and color for a cognitive system based on its health.
fn system_health(soul: &serde_json::Value, name: &str) -> (f64, String) {
    match name {
        "BRAIN" => {
            let b = soul.get("brain");
            let loss = b.and_then(|b| b.get("running_loss")).and_then(|v| v.as_f64()).unwrap_or(1.0);
            let health = 1.0 - loss.min(1.0);
            (10.0 + health * 6.0, health_color(health))
        }
        "CORTEX" => {
            let c = soul.get("cortex");
            let acc = c.and_then(|c| c.get("prediction_accuracy")).and_then(|v| v.as_str())
                .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok()).unwrap_or(0.0) / 100.0;
            (10.0 + acc * 6.0, health_color(acc))
        }
        "GENESIS" => {
            let g = soul.get("genesis");
            let gen = g.and_then(|g| g.get("generation")).and_then(|v| v.as_u64()).unwrap_or(0);
            let health = (gen as f64 / 200.0).min(1.0);
            (10.0 + health * 6.0, health_color(health))
        }
        "HIVEMND" => {
            let h = soul.get("hivemind");
            let trails = h.and_then(|h| h.get("total_trails")).and_then(|v| v.as_u64()).unwrap_or(0);
            let health = (trails as f64 / 100.0).min(1.0);
            (10.0 + health * 6.0, health_color(health))
        }
        "SYNTH" => {
            let s = soul.get("synthesis");
            let state = s.and_then(|s| s.get("state")).and_then(|v| v.as_str()).unwrap_or("--");
            let health = match state { "coherent" | "exploiting" => 0.9, "exploring" => 0.6, "conflicted" => 0.3, "stuck" => 0.1, _ => 0.5 };
            (10.0 + health * 6.0, health_color(health))
        }
        "EVAL" => {
            let e = soul.get("evaluation");
            let records = e.and_then(|e| e.get("total_records")).and_then(|v| v.as_u64()).unwrap_or(0);
            let health = (records as f64 / 100.0).min(1.0);
            (10.0 + health * 6.0, health_color(health))
        }
        "FREE-E" => {
            let fe = soul.get("free_energy");
            let f = fe.and_then(|f| f.get("F")).and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
            let health = 1.0 - f.min(1.0); // Lower F = healthier
            (10.0 + health * 6.0, health_color(health))
        }
        _ => (10.0, "#3a4a3a".to_string()),
    }
}

fn health_color(health: f64) -> String {
    // 0.0 = red, 0.5 = amber, 1.0 = green
    if health > 0.7 {
        format!("rgba(0,255,65,{:.2})", 0.4 + health * 0.4)
    } else if health > 0.4 {
        format!("rgba(255,160,0,{:.2})", 0.4 + health * 0.3)
    } else {
        format!("rgba(255,23,68,{:.2})", 0.3 + health * 0.3)
    }
}
