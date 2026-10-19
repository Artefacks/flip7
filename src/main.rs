//! Feuille de score Flip 7, pensée pour l'écran d'un téléphone.

use flip7::game::GameError;
use flip7::state::{digits_only, format_names, Phase, Saved, Session};
use leptos::prelude::*;

const STORAGE_KEY: &str = "flip7.score.v1";

#[derive(Copy, Clone)]
struct Model {
    session: RwSignal<Session>,
    draft: RwSignal<Vec<String>>,
    name_input: RwSignal<String>,
    target_input: RwSignal<String>,
    error: RwSignal<Option<GameError>>,
    confirm_undo: RwSignal<bool>,
}

impl Model {
    fn save(self) {
        let Some(storage) = local_storage() else {
            return;
        };
        let target_input = self.target_input.get_untracked();
        let draft = self.draft.get_untracked();
        let saved = self
            .session
            .with_untracked(|session| session.to_saved(&target_input, &draft));
        if let Ok(json) = serde_json::to_string(&saved) {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
    }

    fn clear_prompts(self) {
        if self.error.get_untracked().is_some() {
            self.error.set(None);
        }
        if self.confirm_undo.get_untracked() {
            self.confirm_undo.set(false);
        }
    }

    fn set_name(self, value: String) {
        self.name_input.set(value);
        self.clear_prompts();
    }

    fn set_target(self, value: String) {
        self.target_input.set(digits_only(&value, 4));
        self.clear_prompts();
        self.save();
    }

    fn add_player(self) {
        let name = self.name_input.get_untracked();
        let mut outcome = Ok(());
        self.session.update(|session| {
            outcome = session.add_player(&name);
        });
        match outcome {
            Ok(()) => {
                self.name_input.set(String::new());
                self.error.set(None);
            }
            Err(err) => self.error.set(Some(err)),
        }
        self.save();
    }

    fn remove_player(self, name: String) {
        self.session.update(|session| session.remove_player(&name));
        self.error.set(None);
        self.save();
    }

    fn start(self) {
        let target = self.target_input.get_untracked();
        let mut outcome = Ok(());
        self.session.update(|session| {
            outcome = session.start(&target);
        });
        if outcome.is_ok() {
            let count = self.session.with_untracked(|session| session.players.len());
            self.draft.set(vec![String::new(); count]);
            self.error.set(None);
            self.confirm_undo.set(false);
            scroll_top();
        } else if let Err(err) = outcome {
            self.error.set(Some(err));
        }
        self.save();
    }

    fn set_score(self, index: usize, value: String) {
        let clean = digits_only(&value, 3);
        self.draft.update(|draft| {
            if let Some(slot) = draft.get_mut(index) {
                *slot = clean;
            }
        });
        self.clear_prompts();
        self.save();
    }

    fn toggle_zero(self, index: usize) {
        let current = self
            .draft
            .with_untracked(|draft| draft.get(index).cloned().unwrap_or_default());
        let next = if current == "0" {
            String::new()
        } else {
            "0".into()
        };
        self.set_score(index, next);
    }

    fn submit_round(self) {
        let draft = self.draft.get_untracked();
        let mut outcome = Ok(());
        self.session.update(|session| {
            outcome = session.submit_round(&draft);
        });
        if outcome.is_ok() {
            let count = self.session.with_untracked(|session| session.players.len());
            self.draft.set(vec![String::new(); count]);
            self.error.set(None);
            self.confirm_undo.set(false);
            scroll_top();
        } else if let Err(err) = outcome {
            self.error.set(Some(err));
        }
        self.save();
    }

    fn ask_undo(self) {
        if self
            .session
            .with_untracked(|session| session.rounds.is_empty())
        {
            self.error.set(Some(GameError::NothingToUndo));
            return;
        }
        self.error.set(None);
        self.confirm_undo.set(true);
    }

    fn cancel_undo(self) {
        self.confirm_undo.set(false);
    }

    fn undo(self) {
        let mut outcome = Ok(Vec::new());
        self.session.update(|session| {
            outcome = session.undo();
        });
        match outcome {
            Ok(draft) => {
                self.draft.set(draft);
                self.error.set(None);
                self.confirm_undo.set(false);
                scroll_top();
            }
            Err(err) => self.error.set(Some(err)),
        }
        self.save();
    }

    fn finish(self) {
        let mut outcome = Ok(());
        self.session.update(|session| {
            outcome = session.finish();
        });
        if let Err(err) = outcome {
            self.error.set(Some(err));
        } else {
            self.error.set(None);
            self.confirm_undo.set(false);
            scroll_top();
        }
        self.save();
    }

    fn rematch(self) {
        self.session.update(|session| session.rematch());
        let count = self.session.with_untracked(|session| session.players.len());
        self.draft.set(vec![String::new(); count]);
        self.error.set(None);
        self.confirm_undo.set(false);
        scroll_top();
        self.save();
    }

    fn reset(self) {
        self.session.set(Session::default());
        self.draft.set(Vec::new());
        self.name_input.set(String::new());
        self.target_input.set("200".into());
        self.error.set(None);
        self.confirm_undo.set(false);
        scroll_top();
        self.save();
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    let (session, draft, target_input) =
        read_saved().unwrap_or_else(|| (Session::default(), Vec::new(), "200".into()));
    let model = Model {
        session: RwSignal::new(session),
        draft: RwSignal::new(draft),
        name_input: RwSignal::new(String::new()),
        target_input: RwSignal::new(target_input),
        error: RwSignal::new(None),
        confirm_undo: RwSignal::new(false),
    };

    view! {
        <main class="app">
            <header class="brand">
                <span class="mark" aria-hidden="true">"7"</span>
                <div>
                    <h1>"Flip 7"</h1>
                    <p class="tag">"Feuille de score"</p>
                </div>
            </header>
            <Show when=move || model.session.with(|session| session.phase == Phase::Setup)>
                <SetupScreen model=model />
            </Show>
            <Show when=move || model.session.with(|session| session.phase == Phase::Playing)>
                <PlayScreen model=model />
            </Show>
            <Show when=move || model.session.with(|session| session.phase == Phase::Finished)>
                <WinnerScreen model=model />
            </Show>
        </main>
    }
}

#[component]
fn SetupScreen(model: Model) -> impl IntoView {
    view! {
        <div class="stack">
            <section class="paper">
                <h2>"Joueurs"</h2>
                <p class="hint">"Ajoute tout le monde avant de commencer. L'ordre reste celui de la table."</p>
                <form on:submit=move |event| {
                    event.prevent_default();
                    model.add_player();
                }>
                    <div class="add-row">
                        <label class="grow">
                            <span>"Nom"</span>
                            <input
                                prop:value=move || model.name_input.get()
                                on:input=move |event| model.set_name(event_target_value(&event))
                                placeholder="Prénom"
                                autocomplete="off"
                                autocapitalize="words"
                                maxlength="24"
                            />
                        </label>
                        <button type="submit" class="add">"Ajouter"</button>
                    </div>
                </form>
                <p class="count">
                    {move || {
                        match model.session.with(|session| session.players.len()) {
                            0 => "Aucun joueur pour l'instant.".to_string(),
                            1 => "Un joueur. Il en faut au moins deux.".to_string(),
                            count => format!("{count} joueurs."),
                        }
                    }}
                </p>
                <ul class="roster">
                    <For
                        each=move || model.session.with(|session| session.players.clone())
                        key=|name| name.clone()
                        let(name)
                    >
                        <PlayerRow model=model name=name />
                    </For>
                </ul>
            </section>
            <section class="paper">
                <label class="field">
                    <span>"Objectif"</span>
                    <input
                        prop:value=move || model.target_input.get()
                        on:input=move |event| model.set_target(event_target_value(&event))
                        inputmode="numeric"
                        autocomplete="off"
                        maxlength="4"
                    />
                    <span class="quiet">"200 dans une partie classique. Le plus haut score l'emporte."</span>
                </label>
            </section>
            <ErrorNote model=model />
            <button type="button" class="primary" on:click=move |_| model.start()>"Commencer"</button>
        </div>
    }
}

#[component]
fn PlayerRow(model: Model, name: String) -> impl IntoView {
    let label = name.clone();
    let remove_name = name.clone();
    view! {
        <li>
            <span class="seat">
                {move || {
                    model.session.with(|session| {
                        session
                            .players
                            .iter()
                            .position(|player| player == &label)
                            .map(|index| index + 1)
                            .unwrap_or(0)
                    })
                }}
            </span>
            <span class="name">{name}</span>
            <button type="button" class="text-btn" on:click=move |_| model.remove_player(remove_name.clone())>
                "Retirer"
            </button>
        </li>
    }
}

#[component]
fn PlayScreen(model: Model) -> impl IntoView {
    view! {
        <div class="stack">
            <section class="paper">
                {move || {
                    let (round, target) = model.session.with(|session| (session.rounds.len() + 1, session.target));
                    view! {
                        <div class="round-head">
                            <div>
                                <p class="kicker">"Manche"</p>
                                <h2>{round}</h2>
                            </div>
                            <div class="target-chip">
                                <span>"Objectif"</span>
                                <strong>{target}</strong>
                            </div>
                        </div>
                    }
                }}
                <Board model=model />
                <History model=model />
            </section>
            <section class="paper">
                <p class="kicker ink">"Cette manche"</p>
                <h2>"Score annoncé"</h2>
                <p class="hint">"Un éliminé marque 0."</p>
                <div class="scores">
                    <For
                        each=move || model.session.with(|session| session.players.clone())
                        key=|name| name.clone()
                        children=move |name| score_entry(model, name)
                    />
                </div>
            </section>
            <GoalNote model=model />
            <ErrorNote model=model />
            <PlayDock model=model />
        </div>
    }
}

fn score_entry(model: Model, name: String) -> impl IntoView {
    let index = model.session.with_untracked(|session| {
        session
            .players
            .iter()
            .position(|player| player == &name)
            .unwrap_or(0)
    });
    let aria = format!("Score de {name}");
    view! {
        <div class="score">
            <div class="score-head">
                <span class="name">{name}</span>
                <span class="entry-total">
                    <span class="entry-num">
                        {move || {
                            model.session.with(|session| {
                                session.game().totals().get(index).copied().unwrap_or(0)
                            })
                        }}
                    </span>
                    <span class="entry-unit">"pts"</span>
                </span>
            </div>
            <div class="score-row">
                <input
                    type="text"
                    inputmode="numeric"
                    autocomplete="off"
                    maxlength="3"
                    placeholder="Score"
                    aria-label=aria
                    prop:value=move || {
                        model.draft.with(|draft| draft.get(index).cloned().unwrap_or_default())
                    }
                    on:input=move |event| model.set_score(index, event_target_value(&event))
                />
                <button
                    type="button"
                    class=move || {
                        if model.draft.with(|draft| draft.get(index).is_some_and(|value| value == "0")) {
                            "bust is-zero"
                        } else {
                            "bust"
                        }
                    }
                    on:click=move |_| model.toggle_zero(index)
                >
                    "Éliminé"
                </button>
            </div>
        </div>
    }
}

#[component]
fn PlayDock(model: Model) -> impl IntoView {
    view! {
        {move || {
            let reached = model.session.with(|session| {
                let game = session.game();
                game.target_reached() && !game.rounds.is_empty()
            });
            let confirming = model.confirm_undo.get();
            let round_count = model.session.with(|session| session.rounds.len());
            view! {
                <div class="dock">
                    {optional_button(reached, move || {
                        view! {
                            <button type="button" class="primary gold" on:click=move |_| model.finish()>
                                "Annoncer le gagnant"
                            </button>
                        }
                    })}
                    <button type="button" class="primary" on:click=move |_| model.submit_round()>
                        "Valider la manche"
                    </button>
                    {undo_controls(model, confirming, round_count)}
                    {optional_button(!reached, move || {
                        view! {
                            <button type="button" class="linkish" on:click=move |_| model.finish()>
                                "Terminer la partie"
                            </button>
                        }
                    })}
                </div>
            }
        }}
    }
}

fn undo_controls(model: Model, confirming: bool, round_count: usize) -> impl IntoView {
    if confirming {
        view! {
            <div class="confirm">
                <p>
                    "Annuler la manche "
                    {round_count}
                    " ? La saisie en cours sera remplacée par les scores de cette manche."
                </p>
                <button type="button" class="danger" on:click=move |_| model.undo()>"Oui, annuler"</button>
                <button type="button" class="ghost" on:click=move |_| model.cancel_undo()>"Non"</button>
            </div>
        }
        .into_any()
    } else if round_count > 0 {
        view! {
            <button type="button" class="ghost" on:click=move |_| model.ask_undo()>
                "Annuler la dernière manche"
            </button>
        }
        .into_any()
    } else {
        ().into_any()
    }
}

fn optional_button<V>(show: bool, render: impl FnOnce() -> V) -> AnyView
where
    V: IntoView + 'static,
{
    if show {
        render().into_any()
    } else {
        ().into_any()
    }
}

#[component]
fn WinnerScreen(model: Model) -> impl IntoView {
    view! {
        <div class="stack">
            <section class="paper hero">
                {move || {
                    let game = model.session.with(|session| session.game());
                    let winners = game.winners();
                    let best = game.standings().first().map(|row| row.total).unwrap_or(0);
                    let tie = winners.len() > 1;
                    let kicker = if tie {
                        "Égalité"
                    } else if game.target_reached() {
                        "Gagnant"
                    } else {
                        "En tête"
                    };
                    let detail = if tie {
                        format!("{best} points")
                    } else if game.target_reached() {
                        format!("remporte la partie avec {best} points")
                    } else {
                        format!("{best} points · objectif de {} non atteint", game.target)
                    };
                    let title = format_names(&winners);
                    view! {
                        <div>
                            <p class="kicker">{kicker}</p>
                            <h2>{title}</h2>
                            <p class="detail">{detail}</p>
                        </div>
                    }
                }}
            </section>
            <section class="paper">
                <h2>"Classement"</h2>
                <Board model=model />
            </section>
            <ErrorNote model=model />
            <WinnerDock model=model />
        </div>
    }
}

#[component]
fn WinnerDock(model: Model) -> impl IntoView {
    view! {
        {move || {
            let confirming = model.confirm_undo.get();
            let round_count = model.session.with(|session| session.rounds.len());
            view! {
                <div class="dock">
                    <button type="button" class="primary" on:click=move |_| model.rematch()>"Revanche"</button>
                    {undo_controls(model, confirming, round_count)}
                    <button type="button" class="ghost" on:click=move |_| model.reset()>"Nouvelle partie"</button>
                </div>
            }
        }}
    }
}

#[component]
fn Board(model: Model) -> impl IntoView {
    view! {
        <ol class="board">
            <For
                each=move || model.session.with(|session| session.game().standings())
                key=|row| format!("{}-{}-{}-{:?}", row.name, row.rank, row.total, row.last)
                children=move |row| {
                    view! {
                        <StandingRow
                            model=model
                            rank=row.rank
                            name=row.name
                            total=row.total
                            last=row.last
                            at_target=row.at_target
                        />
                    }
                }
            />
        </ol>
    }
}

#[component]
fn StandingRow(
    model: Model,
    rank: usize,
    name: String,
    total: i32,
    last: Option<i32>,
    at_target: bool,
) -> impl IntoView {
    let width = model.session.with_untracked(|session| {
        let percent = if session.target <= 0 {
            0.0
        } else {
            (total as f32 / session.target as f32 * 100.0).min(100.0)
        };
        format!("{percent:.0}%")
    });
    let delta = round_delta(last);
    view! {
        <li class={if at_target { "standing hot" } else { "standing" }}>
            <span class="rank">{rank}</span>
            <span class="who">
                <span class="name">{name}</span>
                <span class="delta">{delta}</span>
                <span class="meter" aria-hidden="true">
                    <span class="meter-fill" style:width=width></span>
                </span>
            </span>
            <span class="total-block">
                <span class="total">{total}</span>
                <span class="entry-unit">"pts"</span>
            </span>
        </li>
    }
}

fn round_delta(last: Option<i32>) -> String {
    match last {
        Some(score) if score > 0 => format!("+{score}"),
        Some(_) => "0".to_string(),
        None => String::new(),
    }
}

#[component]
fn History(model: Model) -> impl IntoView {
    view! {
        <Show when=move || model.session.with(|session| !session.rounds.is_empty())>
            {move || {
                let (players, rounds) = model.session.with(|session| (session.players.clone(), session.rounds.clone()));
                let lines = rounds
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(index, scores)| {
                        let parts = scores
                            .iter()
                            .enumerate()
                            .map(|(seat, score)| {
                                let name = players.get(seat).map(String::as_str).unwrap_or("?");
                                format!("{name} {score}")
                            })
                            .collect::<Vec<_>>()
                            .join(" · ");
                        format!("Manche {} — {parts}", index + 1)
                    })
                    .collect::<Vec<_>>();
                view! {
                    <details>
                        <summary>"Manches précédentes"</summary>
                        <ul class="history">
                            {lines.into_iter().map(|line| view! { <li>{line}</li> }).collect::<Vec<_>>()}
                        </ul>
                    </details>
                }
            }}
        </Show>
    }
}

#[component]
fn GoalNote(model: Model) -> impl IntoView {
    view! {
        {move || {
            let game = model.session.with(|session| session.game());
            if !game.target_reached() || game.rounds.is_empty() {
                return ().into_any();
            }
            let best = game.standings().first().map(|row| row.total).unwrap_or(0);
            let winners = game.winners();
            let text = if winners.len() > 1 {
                format!("Objectif atteint. Égalité entre {} à {best} points.", format_names(&winners))
            } else {
                format!("Objectif atteint. {} mène avec {best} points.", format_names(&winners))
            };
            view! { <p class="note goal">{text}</p> }.into_any()
        }}
    }
}

#[component]
fn ErrorNote(model: Model) -> impl IntoView {
    view! {
        <div aria-live="polite">
            {move || {
                model.error.get().map(|err| view! { <p class="note error" role="alert">{err.to_string()}</p> })
            }}
        </div>
    }
}

fn read_saved() -> Option<(Session, Vec<String>, String)> {
    let json = local_storage()?.get_item(STORAGE_KEY).ok().flatten()?;
    serde_json::from_str::<Saved>(&json)
        .ok()
        .map(Session::from_saved)
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn scroll_top() {
    if let Some(window) = web_sys::window() {
        window.scroll_to_with_x_and_y(0.0, 0.0);
    }
}
