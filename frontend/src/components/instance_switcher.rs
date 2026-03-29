use leptos::prelude::*;
use crate::api::Instance;

#[component]
pub fn InstanceSwitcher(
    instances: Vec<Instance>,
    active_id: String,
    on_switch: Callback<String>,
) -> impl IntoView {
    let items: Vec<_> = instances.into_iter().map(|instance| {
        let id = instance.id.clone();
        let is_active = id == active_id;
        let class = if is_active { "instance-item active" } else { "instance-item" };
        let display_name = instance.username.unwrap_or(instance.name.clone());

        view! {
            <button
                class={class}
                on:click=move |_| on_switch.run(id.clone())
            >
                <span class="instance-name">{display_name}</span>
                <span class="instance-url">{instance.base_url.clone()}</span>
            </button>
        }
    }).collect();

    view! {
        <div class="instance-switcher">
            {items}
        </div>
    }
}
