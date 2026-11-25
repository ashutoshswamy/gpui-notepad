//! Runs the update check in the background and stores the result for the banner.
use super::Notepad;
use crate::update;
use gpui::Context;

impl Notepad {
    /// `manual`: also report "up to date" / failures in the status bar.
    pub fn check_updates(&mut self, manual: bool, cx: &mut Context<Self>) {
        if manual {
            self.status = "Checking for updates…".into();
            cx.notify();
        }
        cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async { update::fetch_latest() }).await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(Some(u)) => {
                        this.status = if manual { format!("Version {} is available", u.version) } else { String::new() };
                        this.update = Some(u);
                    }
                    Ok(None) if manual => this.status = format!("You're up to date ({})", update::CURRENT),
                    Err(e) if manual => this.status = format!("Update check failed: {e}"),
                    _ => {}
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
