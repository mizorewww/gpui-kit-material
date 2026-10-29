//! Explicit native GPU capture for reproducible gallery evidence.
use super::{GALLERY_SECTIONS, Gallery, ThemeMode};
use anyhow::{Context as _, Result, bail, ensure};
use gpui::{App, Entity, Global, Window, px, size};
use std::{cell::Cell, path::PathBuf, process::Command, rc::Rc, time::Duration};

pub const USAGE: &str = "gallery [--capture output.png --section themes|motion|buttons|toggles|loading|icons|labels|feedback|colors|typography\n  --width 960 --height 900 --scale 1 --dark --reduced-motion --settle-ms 800\n  --active --disabled --selected --loading --no-ripple --offset-y 0]";

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
                    options.section = GALLERY_SECTIONS
                        .iter()
                        .position(|(id, _)| *id == name)
                        .with_context(|| format!("Unknown section: {name}"))?;
                }
                "--width" => options.width = value()?.parse()?,
                "--height" => options.height = value()?.parse()?,
                "--scale" => options.scale = value()?.parse()?,
                "--offset-y" => options.offset_y = value()?.parse()?,
                "--settle-ms" => options.settle_ms = value()?.parse()?,
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

pub(super) fn window_size(cx: &App) -> Option<gpui::Size<gpui::Pixels>> {
    cx.try_global::<CaptureOptions>()
        .map(|options| size(px(options.width), px(options.height)))
}

pub(super) fn install(view: &Entity<Gallery>, window: &mut Window, cx: &mut App) {
    let Some(options) = cx.try_global::<CaptureOptions>().cloned() else {
        return;
    };
    cx.set_reduce_motion(options.reduced_motion);
    view.update(cx, |gallery, cx| {
        gallery.reference_mode = if options.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        gallery.scale = options.scale.to_string().into();
        gallery.selected_section = options.section;
        gallery.button_active = options.active;
        gallery.button_effects_disabled = options.disabled;
        gallery.button_selections = [options.selected; 10];
        gallery.button_loading = options.loading;
        gallery.button_ripple = options.ripple;
        cx.notify();
    });
    let view = view.clone();
    // Wait for a real layout before asking the chapter anchor to scroll.
    window.on_next_frame(move |window, _| {
        window.on_next_frame(move |window, cx| {
            let anchor = view.read(cx).section_anchors[options.section].clone();
            anchor.scroll_to(window, cx);
            // Anchor scrolling is itself deferred; capture only after that frame.
            window.on_next_frame(move |window, cx| {
            window.spawn(cx, async move |cx| {
                cx.background_executor().timer(Duration::from_millis(options.settle_ms)).await;
                let result=cx.update(|window, cx| -> Result<()> {
                    let scroll=view.read(cx).scroll.clone();
                    let mut offset=scroll.offset();
                    offset.y -= px(options.offset_y);
                    scroll.set_offset(offset);
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
                        "section":GALLERY_SECTIONS[options.section].0,
                        "requested_viewport":{"width":options.width,"height":options.height},
                        "actual_viewport":{"width":f32::from(viewport.width),"height":f32::from(viewport.height)},
                        "image_pixels":{"width":image.width(),"height":image.height()},
                        "device_scale_factor":window.scale_factor(), "text_scale":options.scale,
                        "theme":"baseline", "mode":if options.dark {"dark"} else {"light"},
                        "reduced_motion":cx.reduce_motion(),"settle_ms":options.settle_ms,
                        "scroll_offset":{"x":f32::from(scroll.offset().x),"y":f32::from(scroll.offset().y)},
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

#[cfg(test)]
mod tests {
    use super::CaptureOptions;
    fn parse(args: &[&str]) -> anyhow::Result<Option<CaptureOptions>> {
        CaptureOptions::parse(args.iter().map(|s| s.to_string()))
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
            "toggles",
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
        ])
        .unwrap()
        .unwrap();
        assert_eq!(
            (
                options.section,
                options.width,
                options.height,
                options.scale,
                options.offset_y
            ),
            (3, 1280., 1000., 1.5, -300.)
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
