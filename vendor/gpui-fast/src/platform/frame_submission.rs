//! Successful-presentation accounting for `PlatformWindow` backends. The rules
//! live here, outside any platform module, so they are tested on every host.
//! See `frame-submission-patch.md`.

/// The monotonic count behind `PlatformWindow::successful_frame_submissions`.
///
/// Owned by the window rather than its renderer: renderers are replaced during
/// device-loss recovery, and GPUI treats a decreasing count as unknown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameSubmissionCounter(u64);

impl FrameSubmissionCounter {
    /// The frames counted so far.
    pub fn count(&self) -> u64 {
        self.0
    }

    /// Records one draw from whether each surface it required presented, and
    /// returns whether it counted. A frame counts once however many surfaces
    /// it has, and only when it had a surface and all of them presented: a
    /// composed frame missing a surface was not what the window drew.
    pub fn record_frame(&mut self, surfaces_presented: impl IntoIterator<Item = bool>) -> bool {
        let mut surfaces = surfaces_presented.into_iter().peekable();
        let counted = surfaces.peek().is_some() && surfaces.all(|presented| presented);
        if counted {
            self.0 = self.0.saturating_add(1);
        }
        counted
    }
}

/// Whether an `IDXGISwapChain::Present` `HRESULT` queued the frame for display.
///
/// Only `S_OK` does. `DXGI_STATUS_OCCLUDED` and the mode-change statuses are
/// success codes, so `HRESULT::ok()` accepts them, but nothing was presented.
pub fn dxgi_present_succeeded(hresult: i32) -> bool {
    hresult == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_only_frames_every_surface_presented() {
        let mut counter = FrameSubmissionCounter::default();
        assert!(counter.record_frame([true]));
        assert_eq!(counter.count(), 1);
        // A skipped, failed or rejected presentation is not a frame.
        assert!(!counter.record_frame([false]));
        assert_eq!(counter.count(), 1);
        // A composed frame counts once, whatever its surface count.
        assert!(counter.record_frame([true, true, true]));
        assert_eq!(counter.count(), 2);
        // One failed surface makes the frame incomplete, wherever it is.
        assert!(!counter.record_frame([false, true]));
        assert!(!counter.record_frame([true, false]));
        assert_eq!(counter.count(), 2);
        // Nothing presented is not a frame.
        assert!(!counter.record_frame([]));
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn dxgi_success_statuses_that_present_nothing_do_not_count() {
        const S_OK: i32 = 0;
        const DXGI_STATUS_OCCLUDED: i32 = 0x087A_0001;
        const DXGI_STATUS_MODE_CHANGED: i32 = 0x087A_0007;
        const DXGI_STATUS_MODE_CHANGE_IN_PROGRESS: i32 = 0x087A_0008;
        const DXGI_ERROR_DEVICE_REMOVED: i32 = 0x887A_0005_u32 as i32;

        assert!(dxgi_present_succeeded(S_OK));
        assert!(!dxgi_present_succeeded(DXGI_STATUS_OCCLUDED));
        assert!(!dxgi_present_succeeded(DXGI_STATUS_MODE_CHANGED));
        assert!(!dxgi_present_succeeded(DXGI_STATUS_MODE_CHANGE_IN_PROGRESS));
        assert!(!dxgi_present_succeeded(DXGI_ERROR_DEVICE_REMOVED));
    }
}
