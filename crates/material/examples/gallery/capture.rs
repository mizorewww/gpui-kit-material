//! Explicit native GPU capture for reproducible gallery evidence.
use super::{Gallery, page::CaptureState, pages::PAGES};
use anyhow::{Context as _, Result, bail, ensure};
use gpui::{App, Entity, Global, ListOffset, ListState, Pixels, Window, px, size};
use std::{cell::Cell, path::PathBuf, process::Command, rc::Rc, time::Duration};

pub const USAGE: &str = "gallery [--capture output.png --section <page id>\n  --width 960 --height 900 --scale 1 --dark --reduced-motion --settle-ms 800\n  --active --disabled --selected --loading --no-ripple --offset-y 0 --settings --fps --vsync on|off]\n\nPage ids, in gallery order:\n  color typography shape motion elevation app-bars badges buttons cards\n  carousel checkboxes chips date-time-pickers dialogs dividers lists\n  loading-progress menu navigation radio-button search sheets sliders\n  snackbar switch tabs text-fields toolbars tooltips";

#[derive(Clone)]
pub struct CaptureOptions {
    output: PathBuf,
    section: usize,
    width: f32,
    height: f32,
    scale: f32,
    dark: bool,
    reduced_motion: bool,
    settle_ms: u64,
    active: bool,
    disabled: bool,
    selected: bool,
    loading: bool,
    ripple: bool,
    offset_y: f32,
    settings: bool,
    fps: bool,
    vsync: Option<bool>,
    completed: Rc<Cell<bool>>,
}
impl Global for CaptureOptions {}
impl CaptureOptions {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Option<Self>> {
        let mut args = args.into_iter().peekable();
        if args.peek().is_none() {
            return Ok(None);
        }
        let mut output = None;
        let mut options = Self {
            output: PathBuf::new(),
            section: 0,
            width: 960.,
            height: 900.,
            scale: 1.,
            dark: false,
            reduced_motion: false,
            settle_ms: 800,
            active: false,
            disabled: false,
            selected: false,
            loading: false,
            ripple: true,
            offset_y: 0.,
            settings: false,
            fps: false,
            vsync: None,
            completed: Rc::new(Cell::new(false)),
        };
        while let Some(flag) = args.next() {
            let mut value = || {
                args.next()
                    .with_context(|| format!("Missing value for {flag}"))
            };
            match flag.as_str() {
                "--capture" => output = Some(PathBuf::from(value()?)),
                "--section" => {
                    let name = value()?;
                    options.section = PAGES
                        .iter()
                        .position(|page| page.info.id == name)
                        .with_context(|| format!("Unknown section: {name}"))?;
                }
                "--width" => options.width = value()?.parse()?,
                "--height" => options.height = value()?.parse()?,
                "--scale" => options.scale = value()?.parse()?,
                "--offset-y" => options.offset_y = value()?.parse()?,
                "--settle-ms" => options.settle_ms = value()?.parse()?,
                "--fps" => options.fps = true,
                "--settings" => options.settings = true,
                "--vsync" => {
                    options.vsync = Some(match value()?.as_str() {
                        "on" => true,
                        "off" => false,
                        _ => bail!("--vsync must be on or off"),
                    })
                }
                "--dark" => options.dark = true,
                "--reduced-motion" => options.reduced_motion = true,
                "--active" => options.active = true,
                "--disabled" => options.disabled = true,
                "--selected" => options.selected = true,
                "--loading" => options.loading = true,
                "--no-ripple" => options.ripple = false,
                _ => bail!("Unknown option: {flag}\n{USAGE}"),
            }
        }
        options.output = output.context("Capture options require --capture output.png")?;
        ensure!(
            options.output.extension().is_some_and(|ext| ext == "png"),
            "Capture output must use .png"
        );
        for (name, value) in [
            ("width", options.width),
            ("height", options.height),
            ("scale", options.scale),
        ] {
            ensure!(
                value.is_finite() && value > 0.,
                "{name} must be finite and positive"
            );
        }
        ensure!(options.offset_y.is_finite(), "offset-y must be finite");
        Ok(Some(options))
    }
    pub fn completed(&self) -> Rc<Cell<bool>> {
        self.completed.clone()
    }
    pub fn initialize(self, cx: &mut App) {
        cx.set_reduce_motion(self.reduced_motion);
        let completed = self.completed.clone();
        cx.on_app_quit(move |_| {
            if !completed.get() {
                eprintln!("Gallery quit before capture completed");
                std::process::exit(1);
            }
            async {}
        })
        .detach();
        cx.set_global(self);
    }
}

pub(super) fn show_fps(cx: &App) -> bool {
    cx.try_global::<CaptureOptions>()
        .is_none_or(|options| options.fps)
}

pub(super) fn window_size(cx: &App) -> Option<gpui::Size<gpui::Pixels>> {
    cx.try_global::<CaptureOptions>()
        .map(|options| size(px(options.width), px(options.height)))
}

pub(super) fn install(view: &Entity<Gallery>, window: &mut Window, cx: &mut App) {
    let Some(options) = cx.try_global::<CaptureOptions>().cloned() else {
        return;
    };
    cx.set_reduce_motion(options.reduced_motion);
    if let Some(enabled) = options.vsync
        && let Err(error) = window.set_vsync(enabled)
    {
        eprintln!("Capture vertical sync setting failed: {error:#}");
        std::process::exit(1);
    }
    view.update(cx, |gallery, cx| {
        gallery.apply_capture(
            options.section,
            options.dark,
            options.scale,
            CaptureState {
                active: options.active,
                disabled: options.disabled,
                selected: options.selected,
                loading: options.loading,
                ripple: options.ripple,
            },
            cx,
        );
    });
    let view = view.clone();
    // Wait for initial layout, then position the logical section row.
    window.on_next_frame(move |window, _| {
        window.on_next_frame(move |window, cx| {
            if options.settings {
                super::settings::open(window, cx);
            }
            view.read(cx).scroll_to_section(options.section);
            window.refresh();
            // Allow the target rows to be measured and painted before settling.
            window.on_next_frame(move |window, cx| {
            window.spawn(cx, async move |cx| {
                let positioned = cx.update(|window, cx| -> Result<()> {
                    let scroll = view.read(cx).list.clone();
                    scroll_measured_offset(&scroll, px(options.offset_y), window, cx)?;
                    window.refresh();
                    window.draw(cx).clear(cx);
                    Ok(())
                });
                match positioned { Ok(Ok(())) => {}, Ok(Err(error)) | Err(error) => {
                    eprintln!("Capture positioning failed: {error:#}");
                    std::process::exit(1);
                } }
                // New rows mount while applying the offset. Their animations
                // receive the full settle interval, just like the section row.
                cx.background_executor().timer(Duration::from_millis(options.settle_ms)).await;
                let result=cx.update(|window, cx| -> Result<()> {
                    let scroll=view.read(cx).list.clone();
                    window.refresh();
                    window.draw(cx).clear(cx);
                    let image=window.render_to_image().context("Native GPU image capture failed")?;
                    if let Some(parent)=options.output.parent().filter(|path| !path.as_os_str().is_empty()) { std::fs::create_dir_all(parent)?; }
                    image.save(&options.output)?;
                    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
                    let git=|args: &[&str]| -> Option<String> {
                        let output=Command::new("git").args(args).current_dir(&root).output().ok()?;
                        output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
                    };
                    let viewport=window.viewport_size();
                    let metadata=serde_json::json!({
                        "renderer":"native GPUI render_to_image", "source_commit":git(&["rev-parse","HEAD"]),
                        "worktree_dirty":git(&["status","--porcelain"]).map(|status| !status.is_empty()),
                        "section":PAGES[options.section].info.id,
                        "requested_viewport":{"width":options.width,"height":options.height},
                        "actual_viewport":{"width":f32::from(viewport.width),"height":f32::from(viewport.height)},
                        "image_pixels":{"width":image.width(),"height":image.height()},
                        "device_scale_factor":window.scale_factor(), "text_scale":options.scale,
                        "theme":"baseline", "mode":if options.dark {"dark"} else {"light"},
                        "fps_overlay":options.fps,
                        "window_submissions":window.submission_stats().count,
                        "settings_open":options.settings,
                        "requested_vsync":options.vsync,
                        "vsync_status":format!("{:?}", window.vsync_status()),
                        "reduced_motion":cx.reduce_motion(),"settle_ms":options.settle_ms,
                        "logical_scroll_top":{"item_ix":scroll.logical_scroll_top().item_ix,"offset_in_item":f32::from(scroll.logical_scroll_top().offset_in_item)},
                        "state":{"active":options.active,"disabled":options.disabled,"selected":options.selected,"loading":options.loading,"ripple":options.ripple},
                        "os":std::env::consts::OS,"architecture":std::env::consts::ARCH,
                        "fonts":"gallery/fonts/roboto (registered 400/500)",
                        "scope":"Rendered gallery evidence; not a pixel-conformance verdict or timed animation sample"
                    });
                    std::fs::write(options.output.with_extension("json"),serde_json::to_vec_pretty(&metadata)?)?;
                    options.completed.set(true);
                    Ok(())
                });
                match result { Ok(Ok(())) => {}, Ok(Err(error)) | Err(error) => {
                    eprintln!("Capture failed: {error:#}");
                    // macOS quit terminates the event loop with status zero.
                    std::process::exit(1);
                } }
                let _=cx.update(|_,cx| cx.quit());
            }).detach();
            });
        });
    });
}

/// Advance using measured row heights. A ListState's unmeasured entries have
/// zero summary height, so pixel scroll_by cannot skip arbitrary unseen rows.
/// This capture-only walk paints each crossed row before consuming its height;
/// performance::prepare rejects combining profiling with --capture.
fn scroll_measured_offset(
    list: &ListState,
    delta: Pixels,
    window: &mut Window,
    cx: &mut App,
) -> Result<()> {
    if delta == px(0.) || list.item_count() == 0 {
        return Ok(());
    }
    let start = list.logical_scroll_top();
    let mut index = start.item_ix.min(list.item_count() - 1);
    let mut offset = start.offset_in_item + delta;
    let measure = |index, window: &mut Window, cx: &mut App| -> Result<Pixels> {
        list.scroll_to(ListOffset {
            item_ix: index,
            offset_in_item: px(0.),
        });
        window.refresh();
        window.draw(cx).clear(cx);
        list.bounds_for_item(index)
            .map(|bounds| bounds.size.height)
            .context("Capture could not measure the target Gallery row")
    };
    if offset < px(0.) {
        while offset < px(0.) && index > 0 {
            index -= 1;
            offset += measure(index, window, cx)?;
        }
        offset = offset.max(px(0.));
    } else {
        loop {
            let height = measure(index, window, cx)?;
            if offset < height {
                break;
            }
            if index + 1 == list.item_count() {
                offset = height;
                break;
            }
            offset -= height;
            index += 1;
        }
    }
    list.scroll_to(ListOffset {
        item_ix: index,
        offset_in_item: offset,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CaptureOptions, PAGES};
    fn parse(args: &[&str]) -> anyhow::Result<Option<CaptureOptions>> {
        CaptureOptions::parse(args.iter().map(|s| s.to_string()))
    }
    #[test]
    fn every_page_is_a_capture_section_named_in_the_usage() {
        for (index, page) in super::PAGES.iter().enumerate() {
            let options = parse(&["--capture", "out.png", "--section", page.info.id])
                .unwrap()
                .unwrap();
            assert_eq!(options.section, index);
            assert!(
                super::USAGE
                    .split_whitespace()
                    .any(|word| word == page.info.id),
                "usage lists {}",
                page.info.id
            );
        }
    }
    #[test]
    fn invalid_capture_requests_fail_instead_of_silently_launching() {
        assert!(parse(&[]).unwrap().is_none());
        for args in [
            vec!["--section", "unknown"],
            vec!["--capture", "out.jpg"],
            vec!["--capture", "out.png", "--scale", "NaN"],
            vec!["--capture", "out.png", "--width", "0"],
            vec!["--capture", "out.png", "--height"],
            vec!["--capture", "out.png", "--vsync", "invalid"],
            vec!["--unknown"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
    }
    #[test]
    fn explicit_capture_configuration_is_preserved() {
        let options = parse(&[
            "--capture",
            "out.png",
            "--section",
            "buttons",
            "--width",
            "1280",
            "--height",
            "1000",
            "--scale",
            "1.5",
            "--dark",
            "--reduced-motion",
            "--selected",
            "--disabled",
            "--active",
            "--loading",
            "--no-ripple",
            "--offset-y",
            "-300",
            "--settings",
            "--fps",
            "--vsync",
            "off",
        ])
        .unwrap()
        .unwrap();
        assert!(options.settings && options.fps);
        assert_eq!(options.vsync, Some(false));
        assert_eq!(
            (
                options.section,
                options.width,
                options.height,
                options.scale,
                options.offset_y
            ),
            (
                PAGES
                    .iter()
                    .position(|page| page.info.id == "buttons")
                    .unwrap(),
                1280.,
                1000.,
                1.5,
                -300.
            )
        );
        assert!(
            options.dark
                && options.reduced_motion
                && options.selected
                && options.disabled
                && options.active
                && options.loading
                && !options.ripple
        );
    }
}

#[cfg(test)]
mod offset_tests {
    use super::scroll_measured_offset;
    use gpui::{
        AppContext, Context, IntoElement, ListAlignment, ListOffset, ListState, Render,
        TestAppContext, Window, div, list, prelude::*, px, size,
    };

    struct VariableRows {
        list: ListState,
    }
    impl Render for VariableRows {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            list(self.list.clone(), |index, _, _| {
                let heights = [40., 200., 80., 140., 300., 100., 500., 120., 200., 100.];
                div().h(px(heights[index])).w_full().into_any_element()
            })
            .size_full()
        }
    }

    #[gpui::test]
    fn capture_offset_measures_unknown_rows_in_both_directions(cx: &mut TestAppContext) {
        let window = cx.open_window(size(px(200.), px(100.)), |_, _| VariableRows {
            list: ListState::new(10, ListAlignment::Top, px(0.)),
        });
        cx.run_until_parked();
        let list = window.update(cx, |view, _, _| view.list.clone()).unwrap();
        list.scroll_to(ListOffset {
            item_ix: 2,
            offset_in_item: px(10.),
        });
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            assert!(
                list.bounds_for_item(4).is_none(),
                "destination must not already be measured/visible"
            );
            scroll_measured_offset(&list, px(400.), window, cx).unwrap();
            window.refresh();
            window.draw(cx).clear(cx);
            let reached = list.logical_scroll_top();
            assert_eq!(reached.item_ix, 4);
            assert_eq!(reached.offset_in_item, px(190.));
            scroll_measured_offset(&list, px(-300.), window, cx).unwrap();
            window.refresh();
            window.draw(cx).clear(cx);
            let returned = list.logical_scroll_top();
            assert_eq!(returned.item_ix, 3);
            assert_eq!(returned.offset_in_item, px(30.));
        })
        .unwrap();
    }
}
