#[cfg(target_os = "windows")]
mod platform {
    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::time::Instant;

    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED,
        COREWEBVIEW2_PROCESS_FAILED_KIND_GPU_PROCESS_EXITED,
        COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED,
        ICoreWebView2Controller, ICoreWebView2ProcessFailedEventArgs,
    };
    use webview2_com::ProcessFailedEventHandler;
    use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        RedrawWindow, RDW_ALLCHILDREN, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    use windows::Win32::System::Power::{
        RegisterPowerSettingNotification, UnregisterPowerSettingNotification, HPOWERNOTIFY,
        POWERBROADCAST_SETTING,
    };
    use windows::Win32::System::RemoteDesktop::{
        WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
    };
    use windows::Win32::System::SystemServices::{
        GUID_CONSOLE_DISPLAY_STATE, GUID_MONITOR_POWER_ON, GUID_SESSION_DISPLAY_STATUS,
    };
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClientRect, IsIconic, IsWindowVisible, KillTimer, PostMessageW, SetTimer,
        DEVICE_NOTIFY_WINDOW_HANDLE, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND,
        PBT_POWERSETTINGCHANGE, SIZE_MAXIMIZED, SIZE_MINIMIZED, SIZE_RESTORED, WA_INACTIVE,
        WM_ACTIVATE, WM_NCDESTROY, WM_POWERBROADCAST, WM_SHOWWINDOW, WM_SIZE, WM_TIMER, WM_USER,
        WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
    };

    use super::{RecoveryDecision, RecoveryTracker, RESTORE_DELAY};

    // Distinct from wry's subclass IDs (WM_USER + 0x64..0x66) so detaching never collides.
    const RECOVERY_SUBCLASS_ID: usize = (WM_USER + 0x200) as usize;
    const WM_CAFFEINE_GPU_RECOVER: u32 = WM_USER + 0x201;
    const TIMER_DEBOUNCE_RECOVER: usize = 0x4341_4601;
    const TIMER_RESTORE_BOUNDS: usize = 0x4341_4602;

    struct SubclassState {
        controller: ICoreWebView2Controller,
        tracker: RefCell<RecoveryTracker>,
        power_notify_handles: Vec<HPOWERNOTIFY>,
        wts_registered: bool,
    }

    pub fn install(window: &tauri::WebviewWindow) {
        let parent_hwnd = match window.hwnd() {
            Ok(hwnd) => hwnd,
            Err(err) => {
                eprintln!("Failed to resolve HWND for WebView2 recovery: {err}");
                return;
            }
        };

        // HWND is !Send because it wraps *mut c_void; forward the integer handle to the UI thread.
        let raw_hwnd = parent_hwnd.0 as isize;
        if let Err(err) = window.with_webview(move |platform_webview| {
            let hwnd = HWND(raw_hwnd as *mut c_void);
            let controller = platform_webview.controller();
            install_on_ui_thread(hwnd, controller);
        }) {
            eprintln!("Failed to schedule WebView2 recovery setup: {err}");
        }
    }

    fn install_on_ui_thread(hwnd: HWND, controller: ICoreWebView2Controller) {
        attach_process_failed_handler(hwnd, &controller);

        let initially_minimized = unsafe { IsIconic(hwnd).as_bool() };
        let initially_visible = unsafe { IsWindowVisible(hwnd).as_bool() };

        let mut power_notify_handles = Vec::with_capacity(3);
        for guid in [
            GUID_CONSOLE_DISPLAY_STATE,
            GUID_MONITOR_POWER_ON,
            GUID_SESSION_DISPLAY_STATUS,
        ] {
            match unsafe {
                RegisterPowerSettingNotification(
                    HANDLE(hwnd.0),
                    &guid,
                    DEVICE_NOTIFY_WINDOW_HANDLE,
                )
            } {
                Ok(handle) => power_notify_handles.push(handle),
                Err(err) => {
                    eprintln!("Failed to register power setting notification {guid:?}: {err}");
                }
            }
        }

        let wts_registered = match unsafe {
            WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION)
        } {
            Ok(()) => true,
            Err(err) => {
                eprintln!("Failed to register session lock notification: {err}");
                false
            }
        };

        let state = Box::new(SubclassState {
            controller,
            tracker: RefCell::new(RecoveryTracker::new(initially_minimized, initially_visible)),
            power_notify_handles,
            wts_registered,
        });
        let state_ptr = Box::into_raw(state);

        let attached = unsafe {
            SetWindowSubclass(
                hwnd,
                Some(recovery_subclass_proc),
                RECOVERY_SUBCLASS_ID,
                state_ptr as usize,
            )
        };
        if !attached.as_bool() {
            let mut failed_state = unsafe { Box::from_raw(state_ptr) };
            unregister_os_notifications(hwnd, &mut failed_state);
            eprintln!("Failed to attach WebView2 recovery window subclass");
        }
    }

    fn attach_process_failed_handler(hwnd: HWND, controller: &ICoreWebView2Controller) {
        let webview = match unsafe { controller.CoreWebView2() } {
            Ok(webview) => webview,
            Err(err) => {
                eprintln!("Failed to access ICoreWebView2 for ProcessFailed hook: {err}");
                return;
            }
        };

        let raw_hwnd = hwnd.0 as isize;
        let handler = ProcessFailedEventHandler::create(Box::new(move |sender, args| {
            let kind = extract_process_failed_kind(args.as_ref());
            if kind == Some(COREWEBVIEW2_PROCESS_FAILED_KIND_GPU_PROCESS_EXITED) {
                eprintln!("WebView2 GPU process exited; queuing compositor surface recovery");
                let target = HWND(raw_hwnd as *mut c_void);
                if let Err(err) = unsafe {
                    PostMessageW(Some(target), WM_CAFFEINE_GPU_RECOVER, WPARAM(0), LPARAM(0))
                } {
                    eprintln!("Failed to post WebView2 GPU recovery message: {err}");
                }
            } else if kind == Some(COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED) {
                eprintln!("WebView2 main render process exited; triggering reload");
                if let Some(webview) = sender {
                    if let Err(err) = unsafe { webview.Reload() } {
                        eprintln!("Failed to reload WebView2 after render process exit: {err}");
                    }
                }
            } else if kind == Some(COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED) {
                eprintln!("WebView2 browser process exited unexpectedly");
            }
            Ok(())
        }));

        let mut token: i64 = 0;
        if let Err(err) = unsafe { webview.add_ProcessFailed(&handler, &mut token) } {
            eprintln!("Failed to subscribe to WebView2 ProcessFailed events: {err}");
        }
    }

    fn extract_process_failed_kind(
        args: Option<&ICoreWebView2ProcessFailedEventArgs>,
    ) -> Option<webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_PROCESS_FAILED_KIND>
    {
        let args = args?;
        let mut kind = Default::default();
        match unsafe { args.ProcessFailedKind(&mut kind) } {
            Ok(()) => Some(kind),
            Err(err) => {
                eprintln!("Failed to inspect WebView2 ProcessFailedKind: {err}");
                None
            }
        }
    }

    unsafe extern "system" fn recovery_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _uid_subclass: usize,
        dw_ref_data: usize,
    ) -> LRESULT {
        let state_ptr = dw_ref_data as *mut SubclassState;
        if state_ptr.is_null() {
            return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        }

        if msg == WM_NCDESTROY {
            unsafe {
                let _ = RemoveWindowSubclass(
                    hwnd,
                    Some(recovery_subclass_proc),
                    RECOVERY_SUBCLASS_ID,
                );
                let _ = KillTimer(Some(hwnd), TIMER_DEBOUNCE_RECOVER);
                let _ = KillTimer(Some(hwnd), TIMER_RESTORE_BOUNDS);
                let mut boxed = Box::from_raw(state_ptr);
                unregister_os_notifications(hwnd, &mut boxed);
                drop(boxed);
                return DefSubclassProc(hwnd, msg, wparam, lparam);
            }
        }

        let state = unsafe { &*(state_ptr as *const SubclassState) };
        let now = Instant::now();

        match msg {
            WM_POWERBROADCAST => unsafe {
                handle_power_broadcast(hwnd, state, wparam, lparam, now);
            },
            WM_WTSSESSION_CHANGE => {
                let code = wparam.0 as u32;
                if code == WTS_SESSION_LOCK {
                    let _ = state
                        .tracker
                        .borrow_mut()
                        .on_session_lock_changed(true, now);
                } else if code == WTS_SESSION_UNLOCK {
                    let decision = state
                        .tracker
                        .borrow_mut()
                        .on_session_lock_changed(false, now);
                    apply_recovery_decision(hwnd, decision);
                }
            }
            WM_SIZE => {
                let size_kind = wparam.0 as u32;
                if size_kind == SIZE_MINIMIZED {
                    let _ = state.tracker.borrow_mut().on_minimized_changed(true, now);
                } else if size_kind == SIZE_RESTORED || size_kind == SIZE_MAXIMIZED {
                    let decision = state.tracker.borrow_mut().on_minimized_changed(false, now);
                    apply_recovery_decision(hwnd, decision);
                }
            }
            WM_SHOWWINDOW => {
                let visible = wparam.0 != 0;
                let decision = state
                    .tracker
                    .borrow_mut()
                    .on_visibility_changed(visible, now);
                apply_recovery_decision(hwnd, decision);
            }
            WM_ACTIVATE => {
                let active = (wparam.0 as u32 & 0xFFFF) != WA_INACTIVE;
                let decision = state
                    .tracker
                    .borrow_mut()
                    .on_activation_changed(active, now);
                apply_recovery_decision(hwnd, decision);
            }
            WM_CAFFEINE_GPU_RECOVER => {
                let decision = state.tracker.borrow_mut().on_gpu_process_failed(now);
                apply_recovery_decision(hwnd, decision);
            }
            WM_TIMER if wparam.0 == TIMER_DEBOUNCE_RECOVER => {
                unsafe {
                    let _ = KillTimer(Some(hwnd), TIMER_DEBOUNCE_RECOVER);
                }
                let should_run = state.tracker.borrow_mut().begin_scheduled_recovery(now);
                if should_run {
                    begin_compositor_nudge(hwnd, &state.controller);
                }
                return LRESULT(0);
            }
            WM_TIMER if wparam.0 == TIMER_RESTORE_BOUNDS => {
                unsafe {
                    let _ = KillTimer(Some(hwnd), TIMER_RESTORE_BOUNDS);
                }
                restore_client_bounds(hwnd, &state.controller);
                return LRESULT(0);
            }
            _ => {}
        }

        unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
    }

    unsafe fn handle_power_broadcast(
        hwnd: HWND,
        state: &SubclassState,
        wparam: WPARAM,
        lparam: LPARAM,
        now: Instant,
    ) {
        let code = wparam.0 as u32;
        if code == PBT_APMRESUMEAUTOMATIC || code == PBT_APMRESUMESUSPEND {
            let decision = state.tracker.borrow_mut().on_system_resume(now);
            apply_recovery_decision(hwnd, decision);
            return;
        }

        if code == PBT_POWERSETTINGCHANGE && lparam.0 != 0 {
            let setting = unsafe { &*(lparam.0 as *const POWERBROADCAST_SETTING) };
            let is_display_guid = setting.PowerSetting == GUID_CONSOLE_DISPLAY_STATE
                || setting.PowerSetting == GUID_MONITOR_POWER_ON
                || setting.PowerSetting == GUID_SESSION_DISPLAY_STATUS;
            if is_display_guid {
                if let Some(display_on) = unsafe { decode_display_on(setting) } {
                    let decision = state
                        .tracker
                        .borrow_mut()
                        .on_display_state_changed(display_on, now);
                    apply_recovery_decision(hwnd, decision);
                }
            }
        }
    }

    unsafe fn decode_display_on(setting: &POWERBROADCAST_SETTING) -> Option<bool> {
        if setting.DataLength as usize >= std::mem::size_of::<u32>() {
            let raw = unsafe { std::ptr::read_unaligned(setting.Data.as_ptr() as *const u32) };
            Some(raw != 0)
        } else if setting.DataLength >= 1 {
            Some(setting.Data[0] != 0)
        } else {
            None
        }
    }

    fn apply_recovery_decision(hwnd: HWND, decision: RecoveryDecision) {
        match decision {
            RecoveryDecision::Ignore | RecoveryDecision::DeferUntilActive => {}
            RecoveryDecision::ScheduleAfter(delay) => {
                let delay_ms = delay.as_millis().clamp(1, u32::MAX as u128) as u32;
                if unsafe { SetTimer(Some(hwnd), TIMER_DEBOUNCE_RECOVER, delay_ms, None) } == 0 {
                    eprintln!("Failed to schedule WebView2 debounce recovery timer");
                }
            }
        }
    }

    // Brief 1 px controller bounds nudge forces WebView2's Viz host to rebuild an evicted swap chain.
    fn begin_compositor_nudge(hwnd: HWND, controller: &ICoreWebView2Controller) {
        if unsafe { IsIconic(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() } {
            return;
        }

        if let Err(err) = unsafe { controller.SetIsVisible(true) } {
            eprintln!("WebView2 recovery SetIsVisible(true) failed: {err}");
        }

        let mut client = RECT::default();
        if let Err(err) = unsafe { GetClientRect(hwnd, &mut client) } {
            eprintln!("WebView2 recovery GetClientRect failed: {err}");
            return;
        }

        let width = client.right - client.left;
        let height = client.bottom - client.top;
        if width <= 2 || height <= 0 {
            return;
        }

        if let Err(err) = unsafe {
            controller.SetBounds(RECT {
                left: 0,
                top: 0,
                right: width - 1,
                bottom: height,
            })
        } {
            eprintln!("WebView2 recovery SetBounds nudge failed: {err}");
            return;
        }

        let restore_ms = RESTORE_DELAY.as_millis().max(1) as u32;
        if unsafe { SetTimer(Some(hwnd), TIMER_RESTORE_BOUNDS, restore_ms, None) } == 0 {
            eprintln!("Failed to schedule WebView2 bounds restore timer; restoring immediately");
            restore_client_bounds(hwnd, controller);
        }
    }

    fn restore_client_bounds(hwnd: HWND, controller: &ICoreWebView2Controller) {
        let mut client = RECT::default();
        if let Err(err) = unsafe { GetClientRect(hwnd, &mut client) } {
            eprintln!("WebView2 bounds restore GetClientRect failed: {err}");
            return;
        }

        let width = client.right - client.left;
        let height = client.bottom - client.top;
        if width <= 0 || height <= 0 {
            return;
        }

        if let Err(err) = unsafe {
            controller.SetBounds(RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            })
        } {
            eprintln!("WebView2 bounds restore SetBounds failed: {err}");
        }

        // Omit RDW_ERASE so Win32 avoids flashing the client background during repaint.
        let _ = unsafe {
            RedrawWindow(
                Some(hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_FRAME,
            )
        };
    }

    fn unregister_os_notifications(hwnd: HWND, state: &mut SubclassState) {
        for handle in state.power_notify_handles.drain(..) {
            if let Err(err) = unsafe { UnregisterPowerSettingNotification(handle) } {
                eprintln!("Failed to unregister power notification: {err}");
            }
        }
        if state.wts_registered {
            if let Err(err) = unsafe { WTSUnRegisterSessionNotification(hwnd) } {
                eprintln!("Failed to unregister session lock notification: {err}");
            }
            state.wts_registered = false;
        }
    }
}

#[cfg(target_os = "windows")]
pub fn install(window: &tauri::WebviewWindow) {
    platform::install(window);
}

#[cfg(not(target_os = "windows"))]
pub fn install(_window: &tauri::WebviewWindow) {}

use std::time::{Duration, Instant};

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const MIN_RECOVERY_INTERVAL: Duration = Duration::from_secs(2);
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const DEBOUNCE_DELAY: Duration = Duration::from_millis(250);
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const RESTORE_DELAY: Duration = Duration::from_millis(32);
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const LONG_INACTIVE_THRESHOLD: Duration = Duration::from_secs(60);

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryDecision {
    Ignore,
    DeferUntilActive,
    ScheduleAfter(Duration),
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Debug)]
pub(crate) struct RecoveryTracker {
    is_minimized: bool,
    is_visible: bool,
    is_locked: bool,
    display_off: bool,
    pending_recovery: bool,
    deactivated_at: Option<Instant>,
    last_recovery_at: Option<Instant>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
impl RecoveryTracker {
    pub(crate) fn new(is_minimized: bool, is_visible: bool) -> Self {
        Self {
            is_minimized,
            is_visible,
            is_locked: false,
            display_off: false,
            pending_recovery: false,
            deactivated_at: None,
            last_recovery_at: None,
        }
    }

    pub(crate) fn on_minimized_changed(&mut self, minimized: bool, now: Instant) -> RecoveryDecision {
        if minimized {
            self.is_minimized = true;
            self.pending_recovery = true;
            return RecoveryDecision::Ignore;
        }
        let was_minimized = std::mem::replace(&mut self.is_minimized, false);
        if was_minimized {
            self.pending_recovery = true;
        }
        self.evaluate_pending(now)
    }

    pub(crate) fn on_visibility_changed(&mut self, visible: bool, now: Instant) -> RecoveryDecision {
        if !visible {
            self.is_visible = false;
            self.pending_recovery = true;
            return RecoveryDecision::Ignore;
        }
        let was_hidden = !std::mem::replace(&mut self.is_visible, true);
        if was_hidden {
            self.pending_recovery = true;
        }
        self.evaluate_pending(now)
    }

    pub(crate) fn on_session_lock_changed(&mut self, locked: bool, now: Instant) -> RecoveryDecision {
        if locked {
            self.is_locked = true;
            self.pending_recovery = true;
            return RecoveryDecision::Ignore;
        }
        let was_locked = std::mem::replace(&mut self.is_locked, false);
        if was_locked {
            self.pending_recovery = true;
        }
        self.evaluate_pending(now)
    }

    pub(crate) fn on_display_state_changed(&mut self, display_on: bool, now: Instant) -> RecoveryDecision {
        if !display_on {
            self.display_off = true;
            self.pending_recovery = true;
            return RecoveryDecision::Ignore;
        }
        let was_off = std::mem::replace(&mut self.display_off, false);
        if was_off {
            self.pending_recovery = true;
        }
        self.evaluate_pending(now)
    }

    pub(crate) fn on_system_resume(&mut self, now: Instant) -> RecoveryDecision {
        self.pending_recovery = true;
        self.evaluate_pending(now)
    }

    pub(crate) fn on_gpu_process_failed(&mut self, now: Instant) -> RecoveryDecision {
        self.pending_recovery = true;
        self.evaluate_pending(now)
    }

    pub(crate) fn on_activation_changed(&mut self, active: bool, now: Instant) -> RecoveryDecision {
        if !active {
            if self.deactivated_at.is_none() {
                self.deactivated_at = Some(now);
            }
            return RecoveryDecision::Ignore;
        }
        if let Some(since) = self.deactivated_at.take() {
            if now.saturating_duration_since(since) >= LONG_INACTIVE_THRESHOLD {
                self.pending_recovery = true;
            }
        }
        self.evaluate_pending(now)
    }

    pub(crate) fn begin_scheduled_recovery(&mut self, now: Instant) -> bool {
        if !self.pending_recovery || !self.can_recover_now() {
            return false;
        }
        self.pending_recovery = false;
        self.last_recovery_at = Some(now);
        true
    }

    fn can_recover_now(&self) -> bool {
        !self.is_minimized && self.is_visible && !self.is_locked && !self.display_off
    }

    fn evaluate_pending(&self, now: Instant) -> RecoveryDecision {
        if !self.pending_recovery {
            return RecoveryDecision::Ignore;
        }
        if !self.can_recover_now() {
            return RecoveryDecision::DeferUntilActive;
        }
        let cooldown_remaining = self
            .last_recovery_at
            .map(|last| MIN_RECOVERY_INTERVAL.saturating_sub(now.saturating_duration_since(last)))
            .unwrap_or(Duration::ZERO);
        RecoveryDecision::ScheduleAfter(cooldown_remaining.max(DEBOUNCE_DELAY))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_wake_while_locked_defers_until_unlock() {
        let t0 = Instant::now();
        let mut tracker = RecoveryTracker::new(false, true);

        assert_eq!(
            tracker.on_display_state_changed(false, t0),
            RecoveryDecision::Ignore
        );
        assert_eq!(
            tracker.on_session_lock_changed(true, t0 + Duration::from_secs(1)),
            RecoveryDecision::Ignore
        );
        assert_eq!(
            tracker.on_display_state_changed(true, t0 + Duration::from_secs(10)),
            RecoveryDecision::DeferUntilActive
        );
        assert_eq!(
            tracker.on_session_lock_changed(false, t0 + Duration::from_secs(11)),
            RecoveryDecision::ScheduleAfter(DEBOUNCE_DELAY)
        );
        assert!(tracker.begin_scheduled_recovery(t0 + Duration::from_secs(11) + DEBOUNCE_DELAY));
    }

    #[test]
    fn restore_from_minimize_respects_cooldown_without_dropping_recovery() {
        let t0 = Instant::now();
        let mut tracker = RecoveryTracker::new(false, true);

        assert_eq!(
            tracker.on_system_resume(t0),
            RecoveryDecision::ScheduleAfter(DEBOUNCE_DELAY)
        );
        let first_exec = t0 + DEBOUNCE_DELAY;
        assert!(tracker.begin_scheduled_recovery(first_exec));

        assert_eq!(
            tracker.on_minimized_changed(true, first_exec + Duration::from_millis(100)),
            RecoveryDecision::Ignore
        );
        let decision =
            tracker.on_minimized_changed(false, first_exec + Duration::from_millis(500));
        assert_eq!(
            decision,
            RecoveryDecision::ScheduleAfter(Duration::from_millis(1500))
        );
    }

    #[test]
    fn short_unfocus_does_not_trigger_bounds_nudge_but_long_unfocus_does() {
        let t0 = Instant::now();
        let mut tracker = RecoveryTracker::new(false, true);

        assert_eq!(
            tracker.on_activation_changed(false, t0),
            RecoveryDecision::Ignore
        );
        assert_eq!(
            tracker.on_activation_changed(true, t0 + Duration::from_secs(5)),
            RecoveryDecision::Ignore
        );

        assert_eq!(
            tracker.on_activation_changed(false, t0 + Duration::from_secs(10)),
            RecoveryDecision::Ignore
        );
        assert_eq!(
            tracker.on_activation_changed(true, t0 + Duration::from_secs(75)),
            RecoveryDecision::ScheduleAfter(DEBOUNCE_DELAY)
        );
    }

    #[test]
    fn redundant_on_events_do_not_schedule_spurious_recovery() {
        let t0 = Instant::now();
        let mut tracker = RecoveryTracker::new(false, true);

        assert_eq!(
            tracker.on_display_state_changed(true, t0),
            RecoveryDecision::Ignore
        );
        assert_eq!(
            tracker.on_minimized_changed(false, t0 + Duration::from_secs(1)),
            RecoveryDecision::Ignore
        );
        assert_eq!(
            tracker.on_visibility_changed(true, t0 + Duration::from_secs(2)),
            RecoveryDecision::Ignore
        );
        assert_eq!(
            tracker.on_session_lock_changed(false, t0 + Duration::from_secs(3)),
            RecoveryDecision::Ignore
        );
    }

    #[test]
    fn gpu_process_exit_schedules_recovery_and_suppresses_duplicate_exec() {
        let t0 = Instant::now();
        let mut tracker = RecoveryTracker::new(false, true);

        assert_eq!(
            tracker.on_gpu_process_failed(t0),
            RecoveryDecision::ScheduleAfter(DEBOUNCE_DELAY)
        );
        assert!(tracker.begin_scheduled_recovery(t0 + DEBOUNCE_DELAY));
        assert!(!tracker.begin_scheduled_recovery(t0 + DEBOUNCE_DELAY));
    }
}
