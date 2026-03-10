use gloo_storage::{LocalStorage, Storage};
use gloo_timers::callback::Timeout;
use js_sys::Math;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Element, MouseEvent, TouchEvent};
use yew::prelude::*;

const HUE_COUNT: usize = 360;
const WINDOW_RADIUS: i32 = 10;
const LOCAL_STORAGE_KEY: &str = "colour_tester_stats";
const MAX_DELTA: f64 = 128.0;
const SAFE_DELTA_CAP: f64 = 90.0; // Keep 2*delta within 180 so shortest-distance logic stays valid.

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Stats {
    correct: Vec<u32>,
    wrong: Vec<u32>,
    correct_delta_sum: Vec<f64>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            correct: vec![0; HUE_COUNT],
            wrong: vec![0; HUE_COUNT],
            correct_delta_sum: vec![0.0; HUE_COUNT],
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

#[derive(Clone)]
struct Trial {
    center_hue: usize,
    left_hue: usize,
    right_hue: usize,
    correct_side: Side,
    delta: u32,
}

fn load_stats() -> Stats {
    LocalStorage::get(LOCAL_STORAGE_KEY).unwrap_or_default()
}

fn save_stats(stats: &Stats) {
    let _ = LocalStorage::set(LOCAL_STORAGE_KEY, stats);
}

fn wrap_hue(hue: i32) -> usize {
    ((hue % HUE_COUNT as i32) + HUE_COUNT as i32) as usize % HUE_COUNT
}

fn window_indices(center: usize) -> Vec<usize> {
    let mut indices = Vec::with_capacity((WINDOW_RADIUS * 2 + 1) as usize);
    for offset in -WINDOW_RADIUS..=WINDOW_RADIUS {
        indices.push(wrap_hue(center as i32 + offset));
    }
    indices
}

fn window_counts(stats: &Stats, center: usize) -> (u32, u32) {
    let indices = window_indices(center);
    let mut r = 0u32;
    let mut w = 0u32;
    for idx in indices {
        r += stats.correct[idx];
        w += stats.wrong[idx];
    }
    (r, w)
}

fn delta_from_counts(r: u32, w: u32) -> u32 {
    let r = r as f64 + 1.0;
    let w = w as f64 + 1.0;
    let denom = (r + w).max(1.0);
    let value = (1.0 - r / denom) * MAX_DELTA - r + w;
    let mut delta = 1.0 + value.max(0.0);
    delta = delta.clamp(1.0, SAFE_DELTA_CAP);
    delta.round() as u32
}

fn average_delta_for_hue(stats: &Stats, center: usize) -> f64 {
    let indices = window_indices(center);
    let mut sum = 0.0;
    let mut count = 0.0;
    for idx in indices {
        sum += stats.correct_delta_sum[idx];
        count += stats.correct[idx] as f64;
    }
    if count > 0.0 {
        sum / count
    } else {
        let (r, w) = window_counts(stats, center);
        delta_from_counts(r, w) as f64
    }
}

fn weighted_center(stats: &Stats) -> usize {
    let mut weights = Vec::with_capacity(HUE_COUNT);
    let mut total = 0.0;
    for hue in 0..HUE_COUNT {
        let (r, w) = window_counts(stats, hue);
        let count = (r + w) as f64;
        let weight = 1.0 / (1.0 + count);
        total += weight;
        weights.push(weight);
    }
    let mut target = Math::random() * total;
    for (idx, weight) in weights.into_iter().enumerate() {
        if target <= weight {
            return idx;
        }
        target -= weight;
    }
    0
}

fn generate_trial(stats: &Stats) -> Trial {
    let center_hue = weighted_center(stats);
    let (r, w) = window_counts(stats, center_hue);
    let delta = delta_from_counts(r, w);
    let close_left = Math::random() < 0.5;
    let close_offset = delta as i32;
    let far_offset = (delta * 2) as i32;
    let left_offset = if close_left { -close_offset } else { -far_offset };
    let right_offset = if close_left { far_offset } else { close_offset };
    let left_hue = wrap_hue(center_hue as i32 + left_offset);
    let right_hue = wrap_hue(center_hue as i32 + right_offset);
    let correct_side = if close_left { Side::Left } else { Side::Right };
    Trial {
        center_hue,
        left_hue,
        right_hue,
        correct_side,
        delta,
    }
}

fn element_center_x(node: &NodeRef) -> Option<f64> {
    node.cast::<Element>().map(|el| {
        let rect = el.get_bounding_client_rect();
        rect.x() + rect.width() / 2.0
    })
}

fn selection_threshold_px(center: &NodeRef, left: &NodeRef, right: &NodeRef) -> f64 {
    if let (Some(center_x), Some(left_x), Some(right_x)) = (
        element_center_x(center),
        element_center_x(left),
        element_center_x(right),
    ) {
        let left_distance = (center_x - left_x).abs();
        let right_distance = (right_x - center_x).abs();
        (left_distance.min(right_distance)) / 2.0
    } else {
        60.0
    }
}

fn touch_x(event: &TouchEvent) -> Option<f64> {
    event
        .touches()
        .item(0)
        .or_else(|| event.changed_touches().item(0))
        .map(|touch| touch.client_x() as f64)
}

#[function_component(App)]
fn app() -> Html {
    let stats = use_state(load_stats);
    let trial = {
        let stats = stats.clone();
        use_state(|| generate_trial(&*stats))
    };
    let drag_offset = use_state(|| 0.0);
    let dragging = use_state(|| false);
    let drag_start_x = use_state(|| 0.0);
    let feedback = use_state(|| None::<bool>);
    let feedback_timer = use_mut_ref(|| None::<Timeout>);

    let left_ref = use_node_ref();
    let center_ref = use_node_ref();
    let right_ref = use_node_ref();

    let on_mouse_down = {
        let dragging = dragging.clone();
        let drag_start_x = drag_start_x.clone();
        let drag_offset = drag_offset.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            dragging.set(true);
            drag_start_x.set(event.client_x() as f64);
            drag_offset.set(0.0);
        })
    };

    let on_mouse_move = {
        let dragging = dragging.clone();
        let drag_offset = drag_offset.clone();
        let drag_start_x = drag_start_x.clone();
        Callback::from(move |event: MouseEvent| {
            if !*dragging {
                return;
            }
            let offset = event.client_x() as f64 - *drag_start_x;
            drag_offset.set(offset);
        })
    };

    let apply_choice = {
        let stats = stats.clone();
        let trial = trial.clone();
        let feedback = feedback.clone();
        let feedback_timer = feedback_timer.clone();
        let drag_offset = drag_offset.clone();
        Callback::from(move |choice: Side| {
            let current_trial = (*trial).clone();
            let is_correct = choice == current_trial.correct_side;
            let mut updated = (*stats).clone();
            if is_correct {
                updated.correct[current_trial.center_hue] += 1;
                updated.correct_delta_sum[current_trial.center_hue] += current_trial.delta as f64;
            } else {
                updated.wrong[current_trial.center_hue] += 1;
            }
            save_stats(&updated);
            stats.set(updated.clone());
            trial.set(generate_trial(&updated));
            drag_offset.set(0.0);

            feedback.set(Some(is_correct));
            if let Some(existing) = feedback_timer.borrow_mut().take() {
                drop(existing);
            }
            let feedback = feedback.clone();
            let timeout = Timeout::new(700, move || {
                feedback.set(None);
            });
            *feedback_timer.borrow_mut() = Some(timeout);
        })
    };

    let on_mouse_up = {
        let dragging = dragging.clone();
        let drag_offset = drag_offset.clone();
        let center_ref = center_ref.clone();
        let left_ref = left_ref.clone();
        let right_ref = right_ref.clone();
        let apply_choice = apply_choice.clone();
        Callback::from(move |_| {
            if !*dragging {
                return;
            }
            dragging.set(false);
            let threshold = selection_threshold_px(&center_ref, &left_ref, &right_ref);
            let offset = *drag_offset;
            if offset <= -threshold {
                apply_choice.emit(Side::Left);
            } else if offset >= threshold {
                apply_choice.emit(Side::Right);
            } else {
                drag_offset.set(0.0);
            }
        })
    };

    let on_touch_start = {
        let dragging = dragging.clone();
        let drag_start_x = drag_start_x.clone();
        let drag_offset = drag_offset.clone();
        Callback::from(move |event: TouchEvent| {
            if let Some(x) = touch_x(&event) {
                event.prevent_default();
                dragging.set(true);
                drag_start_x.set(x);
                drag_offset.set(0.0);
            }
        })
    };

    let on_touch_move = {
        let dragging = dragging.clone();
        let drag_offset = drag_offset.clone();
        let drag_start_x = drag_start_x.clone();
        Callback::from(move |event: TouchEvent| {
            if !*dragging {
                return;
            }
            if let Some(x) = touch_x(&event) {
                event.prevent_default();
                let offset = x - *drag_start_x;
                drag_offset.set(offset);
            }
        })
    };

    let on_touch_end = {
        let dragging = dragging.clone();
        let drag_offset = drag_offset.clone();
        let center_ref = center_ref.clone();
        let left_ref = left_ref.clone();
        let right_ref = right_ref.clone();
        let apply_choice = apply_choice.clone();
        Callback::from(move |_| {
            if !*dragging {
                return;
            }
            dragging.set(false);
            let threshold = selection_threshold_px(&center_ref, &left_ref, &right_ref);
            let offset = *drag_offset;
            if offset <= -threshold {
                apply_choice.emit(Side::Left);
            } else if offset >= threshold {
                apply_choice.emit(Side::Right);
            } else {
                drag_offset.set(0.0);
            }
        })
    };

    let map_values: Vec<f64> = (0..HUE_COUNT)
        .map(|hue| average_delta_for_hue(&*stats, hue))
        .collect();

    let mut path = String::new();
    for (hue, delta) in map_values.iter().enumerate() {
        let normalized = ((delta - 1.0) / (SAFE_DELTA_CAP - 1.0)).clamp(0.0, 1.0);
        let y = 100.0 - normalized * 100.0;
        if hue == 0 {
            path.push_str(&format!("M {} {}", hue, y));
        } else {
            path.push_str(&format!(" L {} {}", hue, y));
        }
    }

    let feedback_text = match *feedback {
        Some(true) => Some(("Correct", "feedback correct")),
        Some(false) => Some(("Wrong", "feedback wrong")),
        None => None,
    };

    let current_trial = &*trial;
    let center_style = format!(
        "background: hsl({}, 100%, 50%); transform: translateX({}px);",
        current_trial.center_hue, *drag_offset
    );
    let left_style = format!("background: hsl({}, 100%, 50%);", current_trial.left_hue);
    let right_style = format!("background: hsl({}, 100%, 50%);", current_trial.right_hue);

    html! {
        <div class="app">
            <section class="play-area"
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up.clone()}
                onmouseleave={on_mouse_up}
                ontouchmove={on_touch_move}
                ontouchend={on_touch_end.clone()}
                ontouchcancel={on_touch_end}
            >
                <div class="headline">
                    <div class="title">{ "Colour Closer" }</div>
                    <div class="subtitle">{ "Drag the center tile toward the closest hue." }</div>
                </div>
                <div class="tile-row">
                    <div class="color-tile side" style={left_style} ref={left_ref} />
                    <div
                        class="color-tile center"
                        style={center_style}
                        ref={center_ref}
                        onmousedown={on_mouse_down}
                        ontouchstart={on_touch_start}
                    >
                        <span class="center-label">{ "drag" }</span>
                    </div>
                    <div class="color-tile side" style={right_style} ref={right_ref} />
                </div>
                {
                    if let Some((text, class_name)) = feedback_text {
                        html! { <div class={class_name}>{ text }</div> }
                    } else {
                        html! {}
                    }
                }
                <div class="meta">
                    <span>{ format!("Delta {}", current_trial.delta) }</span>
                    <span>{ "Hue distance" }</span>
                </div>
            </section>
            <section class="map-area">
                <div class="map-title">{ "Distinguishability by Hue" }</div>
                <div class="map-chart">
                    <svg viewBox="0 0 360 100" preserveAspectRatio="none">
                        <path d={path} />
                    </svg>
                </div>
            </section>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
