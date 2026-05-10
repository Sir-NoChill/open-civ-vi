//! New-game wizard — scaffold port of `hifi/newgame.jsx`. Implements the
//! full 5-step strip (Map / Civilization / Rules / Players / Review) with
//! Map and Review having content; the middle three steps render an empty
//! panel for now (TODO: port StepCiv / StepRules / StepPlayers).

use leptos::prelude::*;

use crate::components::{Btn, MiniMap, Panel, PanelHead, Segmented, segmented::Segment, Toggle};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Step {
    Map,
    Civ,
    Rules,
    Players,
    Review,
}

impl Step {
    const ALL: &'static [Step] = &[Step::Map, Step::Civ, Step::Rules, Step::Players, Step::Review];

    fn label(self) -> &'static str {
        match self {
            Step::Map => "Map",
            Step::Civ => "Civilization",
            Step::Rules => "Rules",
            Step::Players => "Players",
            Step::Review => "Review",
        }
    }

    fn idx(self) -> usize {
        Step::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    fn next(self) -> Option<Step> {
        Step::ALL.get(self.idx() + 1).copied()
    }

    fn prev(self) -> Option<Step> {
        if self.idx() == 0 { None } else { Step::ALL.get(self.idx() - 1).copied() }
    }
}

#[component]
pub fn NewGame() -> impl IntoView {
    let step = RwSignal::new(Step::Map);

    view! {
        <div style="flex:1; display:flex; flex-direction:column; min-height:0">
            <div class="content-header">
                <div class="title">"New game"</div>
                <span class="crumbs">"// procedural worldgen"</span>
                <div class="actions">
                    <Btn variant="ghost" size="sm">"presets"</Btn>
                </div>
            </div>

            <StepStrip step=step />

            <div style="flex:1; overflow:auto; padding-bottom:12px">
                {move || match step.get() {
                    Step::Map => view! { <StepMap /> }.into_any(),
                    Step::Review => view! { <StepReview /> }.into_any(),
                    other => view! { <StepPlaceholder name=other.label() /> }.into_any(),
                }}
            </div>

            <div class="wizard-footer">
                <Btn variant="ghost" size="sm"
                     disabled=Signal::derive(move || step.get().prev().is_none())
                     on_click=Callback::new(move |_| {
                         if let Some(p) = step.get().prev() { step.set(p); }
                     })>
                    "← back"
                </Btn>
                <span>
                    <span class="kbd">"⏎"</span>" next · "
                    <span class="kbd">"⌘K"</span>" jump · "
                    <span class="kbd">"esc"</span>" cancel"
                </span>
                {move || if step.get() == Step::Review {
                    view! { <Btn variant="accent">"⌬ generate"</Btn> }.into_any()
                } else {
                    view! {
                        <Btn variant="primary"
                             on_click=Callback::new(move |_| {
                                 if let Some(n) = step.get().next() { step.set(n); }
                             })>
                            "next →"
                        </Btn>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn StepStrip(step: RwSignal<Step>) -> impl IntoView {
    let total = Step::ALL.len();
    view! {
        <div class="wizard-steps">
            {Step::ALL.iter().copied().enumerate().map(|(i, s)| {
                let cur_idx = move || step.get().idx();
                let class = move || {
                    if i < cur_idx() { "step done" }
                    else if i == cur_idx() { "step current" }
                    else { "step" }
                };
                view! {
                    <button class=class on:click=move |_| step.set(s)>
                        <span class="num">{(i + 1).to_string()}</span>
                        <span>{s.label()}</span>
                    </button>
                    {(i < total - 1).then(|| view! { <span class="arrow">"›"</span> })}
                }
            }).collect::<Vec<_>>()}
            <span style="margin-left:auto" class="muted xsmall">
                {move || format!("step {} of {total}", step.get().idx() + 1)}
            </span>
        </div>
    }
}

#[component]
fn StepPlaceholder(name: &'static str) -> impl IntoView {
    view! {
        <div class="wizard-body">
            <Panel flush=true>
                <PanelHead title=name.to_string() sub="// not yet ported" />
                <div class="panel-body">
                    <p class="muted small">
                        "Port pending — see "
                        <code>"docs/open4x-landing/project/hifi/newgame.jsx"</code>
                        " for the JSX shape this screen needs to match."
                    </p>
                </div>
            </Panel>
        </div>
    }
}

// ─────────────────────────────── Step: map ────────────────────────────────────

#[component]
fn StepMap() -> impl IntoView {
    let map_type = RwSignal::new("continents".to_string());
    let map_size = RwSignal::new("std".to_string());
    let advanced = RwSignal::new(false);

    let map_type_opts = Signal::derive(|| {
        ["continents", "pangaea", "archipelago", "fractal", "custom"]
            .iter().map(|s| Segment::from_str(s)).collect()
    });
    let map_size_opts = Signal::derive(|| {
        ["duel", "tiny", "small", "std", "large", "huge"]
            .iter().map(|s| Segment::from_str(s)).collect()
    });

    view! {
        <div class="wizard-body">
            <Panel flush=true>
                <PanelHead
                    title="Map & world".to_string()
                    sub="// procgen parameters"
                />
                <div class="panel-body">
                    <div class="param-row stack">
                        <div class="label">
                            <span class="trigger" title="Continents · Pangaea · Archipelago · Fractal · Custom">
                                "map type"
                            </span>
                        </div>
                        <div class="control">
                            <Segmented options=map_type_opts value=map_type />
                        </div>
                    </div>

                    <div class="param-row stack">
                        <div class="label">
                            <span class="trigger" title="Tile dimensions of the world. duel 44×26 · tiny 60×38 · small 74×46 · std 84×54 · large 96×60 · huge 106×66">
                                "map size"
                            </span>
                            <span class="muted xsmall" style="text-transform:none; letter-spacing:0; margin-left:6px">
                                "standard · 84×54"
                            </span>
                        </div>
                        <div class="control">
                            <Segmented options=map_size_opts value=map_size />
                        </div>
                    </div>

                    <div class="param-row">
                        <div class="label">"advanced parameters"</div>
                        <div class="control">
                            <Toggle on=advanced
                                    on_change=Callback::new(move |v| advanced.set(v)) />
                        </div>
                        <div class="value muted xsmall">
                            "world age · sea level · temperature · rainfall · resources · seed"
                        </div>
                    </div>
                </div>
            </Panel>

            <div class="col">
                <Panel flush=true>
                    <PanelHead
                        title="Preview".to_string()
                        sub="// regenerable"
                    />
                    <div style="padding:1px">
                        <div class="map-preview">
                            <span class="corner">"// 84×54 · continents · seed 0xCAFE…"</span>
                            <span class="corner tr">"⟳"</span>
                            <MiniMap seed=42 style="position:absolute; inset:0; width:100%; height:100%" />
                        </div>
                    </div>
                    <div class="row between center-y" style="padding:10px 14px; border-top:1px solid var(--hairline); font-size:var(--fs-xs)">
                        <span class="muted">"tiles 4536 · land 47% · climate temperate"</span>
                        <Btn variant="ghost" size="sm">"⟳ regenerate"</Btn>
                    </div>
                </Panel>

                <Panel>
                    <div class="h3" style="margin-bottom:8px">"Hint"</div>
                    <p class="muted small" style="margin:0">
                        "The advanced toggle exposes 12+ procgen parameters. Hover any "
                        "underlined label to see what it does."
                    </p>
                </Panel>
            </div>
        </div>
    }
}

// ────────────────────────────── Step: review ──────────────────────────────────

const REVIEW_ROWS: &[(&str, &str)] = &[
    ("map", "continents · standard · 84×54"),
    ("seed", "0xCAFE·B33F·1A77"),
    ("world", "4 bn yrs · sea 50% · temperate · normal rainfall"),
    ("civilization", "Saladin / Arabia"),
    ("difficulty", "prince · standard speed · ancient era"),
    ("victory", "science · culture · domination · religion · score"),
    ("dynamics", "disasters std · barbs std · 12 city-states · AI balanced"),
    ("players", "1 human + 1 invite pending + 6 AI"),
    ("turn mode", "play-by-turn · invite-only · cross-play"),
];

#[component]
fn StepReview() -> impl IntoView {
    view! {
        <div class="wizard-body">
            <Panel flush=true>
                <PanelHead
                    title="Summary".to_string()
                    sub="// last chance to tweak"
                />
                <div class="panel-body">
                    {REVIEW_ROWS.iter().map(|(k, v)| view! {
                        <div class="param-row">
                            <div class="label">{*k}</div>
                            <div class="control" style="font-size:var(--fs-sm)">{*v}</div>
                            <div class="value">
                                <Btn variant="bare" size="xs">"edit"</Btn>
                            </div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </Panel>

            <div class="col">
                <Panel flush=true>
                    <PanelHead
                        title="Final preview".to_string()
                        sub="// world will be locked at generate"
                    />
                    <div style="padding:1px">
                        <div class="map-preview">
                            <MiniMap seed=42 style="position:absolute; inset:0; width:100%; height:100%" />
                        </div>
                    </div>
                </Panel>
                <Panel class="">
                    <p class="small" style="margin-top:0; margin-bottom:12px">
                        "Generation deterministically builds the world from your seed. "
                        "You can copy the seed to recreate this exact map elsewhere."
                    </p>
                    <Btn variant="accent" size="lg" class="block">"⌬  Generate world"</Btn>
                    <p class="muted xsmall" style="text-align:center; margin-top:10px; margin-bottom:0">
                        "// calls /api/games · returns game_id · routes you to gameplay client"
                    </p>
                </Panel>
            </div>
        </div>
    }
}
