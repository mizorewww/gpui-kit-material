//! The example app on the Examples page: one running app shown on the
//! device and screen the configuration panel selects. The app owns all of
//! its state (selections, playback, messages, back stack); the panel's
//! "Screen" control is the one source of truth for what is on screen, so
//! navigating inside the app moves the control and the control moves the app.
//!
//! A device renders at its real size in design px, scaled as a whole to fit
//! the preview by laying it out at a smaller rem ([`spec::RemScaled`]): hit
//! testing, overlays and menus all stay in step with what is painted.
use super::super::super::playground::{PlaygroundState, Value};
use super::super::prelude::*;
use super::parts;
use gpui_base::input::{InputEvent, InputState};
use gpui_kit_material::tokens::typography::BASE_PX;
use std::{collections::HashSet, sync::OnceLock, time::Duration};

/// The kit's example screens, in the panel's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    Home,
    Upcoming,
    Detailed,
    Reviews,
    Library,
    Gallery,
    Messaging,
}

impl Body {
    pub const ALL: [Self; 7] = [
        Self::Home,
        Self::Upcoming,
        Self::Detailed,
        Self::Reviews,
        Self::Library,
        Self::Gallery,
        Self::Messaging,
    ];
    /// The panel's "Screen" options, parallel to [`Self::ALL`].
    pub const NAMES: [&'static str; 7] = [
        "Home",
        "Upcoming",
        "Detailed",
        "Reviews",
        "Library",
        "Gallery",
        "Messaging",
    ];

    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|body| *body == self)
            .expect("every body is listed")
    }

    /// The URL path the browser's address field shows.
    pub fn path(self) -> &'static str {
        match self {
            Self::Home => "www.url.com/home",
            Self::Upcoming => "www.url.com/upcoming",
            Self::Detailed => "www.url.com/detail",
            Self::Reviews => "www.url.com/reviews",
            Self::Library => "www.url.com/library",
            Self::Gallery => "www.url.com/gallery",
            Self::Messaging => "www.url.com/messages",
        }
    }
}

/// The form factors the app runs on. Phone and Tablet are the kit's compact
/// (412dp) and expanded (905×680) frames; Desktop is a large window (M3's
/// 1200dp class) in the kit's browser chrome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Phone,
    Tablet,
    Desktop,
}

impl Device {
    pub const ALL: [Self; 3] = [Self::Phone, Self::Tablet, Self::Desktop];
    pub const NAMES: [&'static str; 3] = ["Phone", "Tablet", "Desktop"];

    /// The screen's size in dp.
    pub const fn screen(self) -> (f32, f32) {
        match self {
            Self::Phone => (412., 917.),
            Self::Tablet => (905., 680.),
            Self::Desktop => (1280., 800.),
        }
    }

    /// The device's outer size in dp, stroke included.
    pub const fn outer(self) -> (f32, f32) {
        let (width, height) = self.screen();
        (
            width + 2. * parts::FRAME_STROKE,
            height + 2. * parts::FRAME_STROKE,
        )
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// The least a device scales to for height alone: below this a window too
/// short for it scrolls the page instead of shrinking it further. Width has
/// no such floor, since the page never scrolls sideways.
const MIN_HEIGHT_SCALE: f32 = 0.6;
/// Stack the panes when that is the only way for a device to stay above this.
const COMFORTABLE_SCALE: f32 = 0.65;

/// How a device fits the preview.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub scale: f32,
    /// Stack the configuration card under the preview to give it the width.
    pub stacked: bool,
}

fn scale_in((width, height): (f32, f32), (outer_width, outer_height): (f32, f32)) -> f32 {
    (width / outer_width)
        .min((height / outer_height).max(MIN_HEIGHT_SCALE))
        .min(1.)
}

/// Fit a device of `outer` size into the canvas, given its area with the panes
/// side by side and stacked.
pub fn fit(outer: (f32, f32), beside: (f32, f32), stacked: (f32, f32)) -> Fit {
    let scale = scale_in(beside, outer);
    let wider = scale_in(stacked, outer);
    if scale < COMFORTABLE_SCALE && wider > scale {
        Fit {
            scale: wider,
            stacked: true,
        }
    } else {
        Fit {
            scale,
            stacked: false,
        }
    }
}

/// The destinations of the navigation bars and rails.
pub const NAV: [(Body, &str, &str, &str); 4] = [
    (Body::Home, "home", "Home", "home"),
    (Body::Library, "library", "Library", "library_music"),
    (Body::Gallery, "gallery", "Gallery", "photo_library"),
    (Body::Reviews, "reviews", "Reviews", "reviews"),
];

pub fn nav_items() -> &'static [NavItem] {
    static ITEMS: OnceLock<Vec<NavItem>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        NAV.iter()
            .map(|(_, value, label, icon)| NavItem::new(*value, *label, parts::nav_icon(icon)))
            .collect()
    })
}

/// The selected destination's value; a screen pushed over them has none.
pub fn nav_value(body: Body) -> &'static str {
    NAV.iter()
        .find(|(destination, ..)| *destination == body)
        .map_or("", |(_, value, ..)| *value)
}

/// Where the rendering screen sits and what it is.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    /// The device's index, which namespaces element ids.
    pub k: usize,
    pub expanded: bool,
    /// The content's width in dp: the phone, or the pane beside the rail.
    pub width: f32,
    /// How far the screen has scrolled under its app bar, in dp.
    pub offset: f32,
}

impl Ctx {
    /// The content's side inset: 16dp compact, 24dp expanded.
    pub fn inset(self) -> f32 {
        if self.expanded { 24. } else { 16. }
    }
}

/// What a screen draws inside its device.
pub struct Screen {
    /// The app bar over the scrolling content, and the room it needs there.
    pub bar: Option<(TopAppBar, f32)>,
    /// Scrolls under the bar.
    pub content: AnyElement,
    /// Pinned under the content: a player or a message composer.
    pub footer: Option<AnyElement>,
    /// Floats over the content: an expanded player.
    pub overlay: Option<AnyElement>,
}

/// How long a message stays on the snackbar.
const SNACKBAR_DURATION: Duration = Duration::from_secs(4);
/// The playback position's step, every tick.
const PLAYBACK_TICK: Duration = Duration::from_millis(500);
const PLAYBACK_STEP: f32 = 0.01;
/// The conversations of the expanded Messaging screen.
pub const CONVERSATIONS: usize = 8;

pub struct ExamplesApp {
    /// The app bar's collapse offset last drawn, in dp.
    pub(crate) bar_offset: f32,
    playground: WeakEntity<PlaygroundState>,
    /// The screen on the device, and the browser's history around it.
    shown: Option<Body>,
    back: Vec<Body>,
    forward: Vec<Body>,
    traversing: bool,
    /// The navigation rail's state; `None` until the user toggles it: a
    /// desktop window starts expanded, a tablet collapsed.
    pub(crate) rail_expanded: Option<bool>,
    /// The one open popup menu, by key.
    pub(crate) open_menu: Option<&'static str>,
    /// Selected filter chips (Library), one bit per chip.
    pub(crate) filters: u16,
    /// Newest or oldest first (Library).
    pub(crate) oldest_first: bool,
    /// The grid or the list view (Library).
    pub(crate) list_view: bool,
    /// The sort of the places (Reviews).
    pub(crate) segment: usize,
    /// Checked list items (Upcoming), one bit per item.
    pub(crate) selected: u8,
    /// Favorited places (Reviews), one bit per item.
    pub(crate) favorites: u8,
    /// The app bar actions toggled on, by name.
    pub(crate) marks: HashSet<&'static str>,
    /// The picked photo (Gallery).
    pub(crate) picked: Option<usize>,
    /// The current track and whether it plays.
    pub(crate) track: usize,
    pub(crate) playing: bool,
    pub(crate) position: f32,
    ticker: Option<Task<()>>,
    /// The selected conversation, its message bar and what was sent to each.
    pub(crate) conversation: usize,
    pub(crate) message: Entity<InputState>,
    pub(crate) sent: [Vec<SharedString>; CONVERSATIONS],
    /// The message on the snackbar and the timer hiding it.
    pub(crate) snackbar: Option<SharedString>,
    snackbar_timer: Option<Task<()>>,
    /// Each screen's scroll position.
    pub(crate) scrolls: [ScrollHandle; Body::ALL.len()],
    _subscription: Subscription,
}

impl ExamplesApp {
    pub fn new(
        playground: WeakEntity<PlaygroundState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let message = cx.new(|cx| InputState::new(window, cx).placeholder("Message"));
        let subscription = cx.subscribe_in(
            &message,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.send(window, cx);
                }
            },
        );
        Self {
            bar_offset: 0.,
            playground,
            shown: None,
            back: Vec::new(),
            forward: Vec::new(),
            traversing: false,
            rail_expanded: None,
            open_menu: None,
            // The kit selects the second chip.
            filters: 0b10,
            oldest_first: false,
            list_view: false,
            segment: 0,
            selected: 0,
            favorites: 0,
            marks: HashSet::new(),
            picked: None,
            track: 0,
            playing: false,
            position: 0.2,
            ticker: None,
            conversation: 0,
            message,
            sent: Default::default(),
            snackbar: None,
            snackbar_timer: None,
            scrolls: Default::default(),
            _subscription: subscription,
        }
    }

    /// Record a screen change, whichever side it came from.
    fn sync_shown(&mut self, body: Body) {
        let previous = self.shown.replace(body);
        if std::mem::take(&mut self.traversing) {
            return;
        }
        if let Some(previous) = previous.filter(|previous| *previous != body) {
            self.back.push(previous);
            self.forward.clear();
        }
    }

    fn set_screen(&self, body: Body, window: &mut Window, cx: &mut Context<Self>) {
        let _ = self.playground.update(cx, |state, cx| {
            state.set_value("screen", Value::Choice(body.index()), window, cx)
        });
    }

    /// Navigate to `body`, as a destination or a tapped item does.
    pub fn go(&mut self, body: Body, window: &mut Window, cx: &mut Context<Self>) {
        self.open_menu = None;
        self.set_screen(body, window, cx);
    }

    /// Go back one screen; with nothing to go back to, to Home.
    pub fn go_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let target = self.back.pop().unwrap_or(Body::Home);
        if let Some(shown) = self.shown.filter(|shown| *shown != target) {
            self.forward.push(shown);
        }
        self.traversing = true;
        self.go(target, window, cx);
    }

    pub fn go_forward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.forward.pop() else {
            return;
        };
        if let Some(shown) = self.shown {
            self.back.push(shown);
        }
        self.traversing = true;
        self.go(target, window, cx);
    }

    pub(crate) fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Scroll the current screen back to its top, as a reload does.
    pub(crate) fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(shown) = self.shown {
            self.scrolls[shown.index()].set_offset(point(px(0.), px(0.)));
        }
        cx.notify();
    }

    /// Show `message` on the snackbar for a few seconds.
    pub fn say(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.snackbar = Some(message.into());
        self.snackbar_timer = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SNACKBAR_DURATION).await;
            let _ = this.update(cx, |this, cx| {
                this.snackbar = None;
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(crate) fn dismiss_snackbar(&mut self, cx: &mut Context<Self>) {
        self.snackbar = None;
        self.snackbar_timer = None;
        cx.notify();
    }

    /// Toggle an app bar action, by name.
    pub(crate) fn toggle_mark(&mut self, name: &'static str, cx: &mut Context<Self>) {
        if !self.marks.remove(name) {
            self.marks.insert(name);
        }
        cx.notify();
    }

    /// Play `track`, or pause it if it is the one playing.
    pub fn play(&mut self, track: usize, cx: &mut Context<Self>) {
        if self.track != track {
            self.track = track;
            self.position = 0.;
            self.set_playing(true, cx);
        } else {
            self.set_playing(!self.playing, cx);
        }
        cx.notify();
    }

    /// The next of the example's three tracks.
    pub fn next_track(&mut self, cx: &mut Context<Self>) {
        self.track = (self.track + 1) % 3;
        self.position = 0.;
        cx.notify();
    }

    fn set_playing(&mut self, playing: bool, cx: &mut Context<Self>) {
        self.playing = playing;
        self.ticker = playing.then(|| {
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(PLAYBACK_TICK).await;
                    let advanced = this.update(cx, |this, cx| {
                        this.position += PLAYBACK_STEP;
                        if this.position >= 1. {
                            this.next_track(cx);
                        } else if this.shown == Some(Body::Detailed) {
                            // Only the Detailed view's player shows the position.
                            cx.notify();
                        }
                    });
                    if advanced.is_err() {
                        break;
                    }
                }
            })
        });
    }

    /// Send the message bar's text to the selected conversation.
    pub fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.message.read(cx).value().trim().to_string();
        if text.is_empty() {
            return;
        }
        self.message
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.send_text(text, cx);
    }

    /// Send `text` to the selected conversation and show it.
    pub fn send_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.sent[self.conversation].push(text.into());
        self.scrolls[Body::Messaging.index()].scroll_to_bottom();
        cx.notify();
    }

    /// Render `body` on `device`, scaled to `scale`.
    pub fn render_device(
        &mut self,
        device: Device,
        body: Body,
        scale: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.sync_shown(body);
        let rem = window.rem_size() * scale;
        // Everything built here reads the device's rem, as it lays out in it.
        let frame = window.with_rem_size(Some(rem), |window| match device {
            Device::Phone => self.phone(body, window, cx),
            Device::Tablet | Device::Desktop => self.expanded(device, body, window, cx),
        });
        spec::RemScaled::new(rem, frame).into_any_element()
    }

    /// `body`'s scroll offset in the device's design px.
    pub(crate) fn offset(&self, body: Body, window: &Window) -> f32 {
        let scrolled = -f32::from(self.scrolls[body.index()].offset().y);
        (scrolled * BASE_PX / f32::from(window.rem_size())).max(0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn a_device_keeps_its_real_size_when_it_fits() {
        let area = (2000., 2000.);
        assert_eq!(
            fit(Device::Phone.outer(), area, area),
            Fit {
                scale: 1.,
                stacked: false
            }
        );
    }

    #[test]
    fn a_device_scales_to_the_width_it_has() {
        // 412 + 16 = 428dp wide into 214dp.
        let area = (214., 2000.);
        assert_eq!(fit(Device::Phone.outer(), area, area).scale, 0.5);
    }

    #[test]
    fn a_short_window_scrolls_the_page_rather_than_shrink_a_device_further() {
        let area = (2000., 300.);
        assert_eq!(
            fit(Device::Phone.outer(), area, area).scale,
            MIN_HEIGHT_SCALE
        );
    }

    #[test]
    fn a_wide_device_takes_the_width_of_the_stacked_panes() {
        let outer = Device::Desktop.outer();
        let fit = fit(outer, (540., 700.), (904., 700.));
        assert!(fit.stacked);
        assert_eq!(fit.scale, 904. / outer.0);
        // A phone is comfortable beside the configuration card.
        let phone = super::fit(Device::Phone.outer(), (540., 700.), (904., 700.));
        assert!(!phone.stacked);
    }

    #[test]
    fn body_names_and_paths_are_parallel_and_distinct() {
        assert_eq!(Body::ALL.len(), Body::NAMES.len());
        let mut paths: Vec<_> = Body::ALL.iter().map(|body| body.path()).collect();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), Body::ALL.len());
    }

    #[test]
    fn destinations_map_back_to_their_values() {
        assert_eq!(nav_value(Body::Library), "library");
        assert_eq!(nav_value(Body::Messaging), "");
        assert_eq!(nav_items().len(), NAV.len());
    }
}
