use notify_rust::{Notification, Timeout};

const APP_NAME: &str = "GUI Scale Applet";
const ICON: &str = "tailscale-icon";

/// Send a desktop notification.
fn send_notification(summary: &str, body: &str) {
    if let Err(e) = Notification::new()
        .appname(APP_NAME)
        .summary(summary)
        .body(body)
        .icon(ICON)
        .timeout(Timeout::Milliseconds(5000))
        .show()
    {
        eprintln!("Failed to show notification: {e}");
    }
}

/// Notify when the Tailscale connection status changes.
pub fn notify_connection_change(connected: bool) {
    let (summary, body) = if connected {
        ("Tailscale Connected", "Your device is now connected to the tailnet.")
    } else {
        ("Tailscale Disconnected", "Your device has been disconnected from the tailnet.")
    };
    send_notification(summary, body);
}

/// Notify when a new device appears on the tailnet.
pub fn notify_new_device(device_name: &str) {
    send_notification(
        "New Device on Tailnet",
        &format!("{device_name} has joined the network."),
    );
}

/// Notify when an incoming file transfer is waiting.
pub fn notify_incoming_files() {
    send_notification(
        "Incoming Tail Drop",
        "File(s) are waiting in your Tail Drop inbox.",
    );
}

/// Notify when file(s) have been sent successfully.
pub fn notify_files_sent(device: &str, count: usize) {
    send_notification(
        "Files Sent",
        &format!("{count} file(s) sent to {device} successfully."),
    );
}

/// Notify when file(s) have been received successfully.
pub fn notify_files_received(path: &str) {
    send_notification(
        "Files Received",
        &format!("File(s) received and saved to {path}."),
    );
}

/// Notify when an account switch occurs.
pub fn notify_account_switched(account: &str) {
    send_notification(
        "Account Switched",
        &format!("Switched to Tailscale account: {account}"),
    );
}
