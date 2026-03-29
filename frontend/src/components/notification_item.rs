use leptos::prelude::*;
use crate::api::Notification;

#[component]
pub fn NotificationItem(notification: Notification) -> impl IntoView {
    let unread_class = if notification.unread { "notification unread" } else { "notification" };

    view! {
        <div class={unread_class}>
            <span class="notification-type">{notification.subject.kind.clone()}</span>
            <span class="notification-title">{notification.subject.title.clone()}</span>
        </div>
    }
}
