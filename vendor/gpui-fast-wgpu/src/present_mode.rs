//! What each `wgpu::PresentMode` means for presentation synchronization. See
//! `MATERIAL-PATCHES.md`.

use wgpu::PresentMode;

/// Whether a present mode synchronizes presentation with the display.
pub fn present_mode_is_synchronized(mode: PresentMode) -> bool {
    match mode {
        PresentMode::AutoVsync | PresentMode::Fifo | PresentMode::FifoRelaxed => true,
        PresentMode::AutoNoVsync | PresentMode::Immediate | PresentMode::Mailbox => false,
    }
}

/// The present mode to use for a synchronization setting among those a surface
/// supports, or `None` when it supports no such mode.
///
/// Unsynchronized prefers `Immediate`, which never waits for the display, over
/// `Mailbox`, which does not wait either but discards superseded frames in the
/// swap chain.
pub fn present_mode_for_synchronization(
    synchronized: bool,
    supported: &[PresentMode],
) -> Option<PresentMode> {
    let preference: &[PresentMode] = if synchronized {
        &[PresentMode::Fifo, PresentMode::FifoRelaxed]
    } else {
        &[PresentMode::Immediate, PresentMode::Mailbox]
    };
    preference
        .iter()
        .copied()
        .find(|mode| supported.contains(mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_report_their_synchronization() {
        for mode in [
            PresentMode::AutoVsync,
            PresentMode::Fifo,
            PresentMode::FifoRelaxed,
        ] {
            assert!(present_mode_is_synchronized(mode), "{mode:?}");
        }
        for mode in [
            PresentMode::AutoNoVsync,
            PresentMode::Immediate,
            PresentMode::Mailbox,
        ] {
            assert!(!present_mode_is_synchronized(mode), "{mode:?}");
        }
    }

    #[test]
    fn selection_honors_preference_and_surface_support() {
        use PresentMode::*;
        assert_eq!(
            present_mode_for_synchronization(false, &[Fifo, Mailbox, Immediate]),
            Some(Immediate)
        );
        assert_eq!(
            present_mode_for_synchronization(false, &[Fifo, Mailbox]),
            Some(Mailbox)
        );
        assert_eq!(present_mode_for_synchronization(false, &[Fifo]), None);
        assert_eq!(
            present_mode_for_synchronization(true, &[Mailbox, FifoRelaxed, Fifo]),
            Some(Fifo)
        );
        assert_eq!(
            present_mode_for_synchronization(true, &[Mailbox, FifoRelaxed]),
            Some(FifoRelaxed)
        );
        assert_eq!(present_mode_for_synchronization(true, &[Mailbox]), None);
        assert_eq!(present_mode_for_synchronization(true, &[]), None);
    }

    #[test]
    fn selected_modes_report_the_requested_synchronization() {
        use PresentMode::*;
        let all = [Fifo, FifoRelaxed, Immediate, Mailbox];
        for synchronized in [true, false] {
            let mode = present_mode_for_synchronization(synchronized, &all).unwrap();
            assert_eq!(present_mode_is_synchronized(mode), synchronized);
        }
    }
}
