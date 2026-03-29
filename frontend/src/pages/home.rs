use leptos::prelude::*;
use crate::api::{self, NoArgs, Notification};
use crate::components::notification_item::NotificationItem;

#[component]
pub fn Home() -> impl IntoView {
    let notifications = LocalResource::new(move || async move {
        api::tauri_invoke::<_, Vec<Notification>>("list_notifications", &NoArgs {}).await
    });

    view! {
        <div class="home-page">
            <h1>"Dashboard"</h1>

            <section class="notifications-section">
                <h2>"Notifications"</h2>
                <Suspense fallback=move || view! { <p>"Loading notifications..."</p> }>
                    {move || notifications.get().map(|result| match (*result).clone() {
                        Ok(items) => {
                            if items.is_empty() {
                                view! { <p class="empty-state">"No new notifications"</p> }.into_any()
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
            </section>
        </div>
    }
}
