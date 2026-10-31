//! Desktop entrypoint for the shared Material gallery.
extern crate gpui_kit as gpui;

#[path = "gallery/mod.rs"]
mod gallery;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("{}", gallery::capture::USAGE);
        return Ok(());
    }
    let capture = gallery::capture::CaptureOptions::parse(args)?;
    let completed = capture.as_ref().map(|options| options.completed());
    gallery::run(gpui_kit::application(), move |cx| {
        if let Some(options) = capture {
            options.initialize(cx);
        }
    });
    if completed.is_some_and(|completed| !completed.get()) {
        anyhow::bail!("Gallery capture did not complete");
    }
    Ok(())
}
