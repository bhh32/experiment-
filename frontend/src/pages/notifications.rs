use leptos::prelude::*;
use crate::api::{self, NoArgs, Notification};
use crate::components::notification_item::NotificationItem;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn Notifications() -> impl IntoView {
    let notifications = LocalResource::new(move || async move {
        api::tauri_invoke::<_, Vec<Notification>>("list_notifications", &NoArgs {}).await
    });

    let mark_all_read = move |_| {
        spawn_local(async move {
            let _ = api::tauri_invoke::<_, ()>("mark_read", &NoArgs {}).await;
        });
    };

    view! {
        <div class="notifications-page">
            <div class="notifications-header">
                <h1>"Notifications"</h1>
                <button class="mark-read-button" on:click=mark_all_read>
                    "Mark all read"
                </button>
            </div>

            <Suspense fallback=move || view! { <p>"Loading notifications..."</p> }>
                {move || notifications.get().map(|result| match (*result).clone() {
                    Ok(items) => {
                        if items.is_empty() {
                            view! { <p class="empty-state">"All caught up!"</p> }.into_any()
                        } else {
                            let views: Vec<_> = items.into_iter().map(|n| {
                                view! { <NotificationItem notification={n} /> }
                            }).collect();
                            view! { <div class="notification-list">{views}</div> }.into_any()
                        }
                    }
                    Err(e) => view! { <p class="error-message">{e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
