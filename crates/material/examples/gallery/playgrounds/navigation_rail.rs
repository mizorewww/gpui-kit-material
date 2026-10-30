//! Placeholder playground, replaced by the component's md3.io playground.
use super::prelude::*;

pub static PLAYGROUND: Playground = Playground {
    scenarios: &[],
    sections: &[],
    preview,
};

fn preview(
    _: &Config,
    _: &mut Preview,
    _: &mut Window,
    _: &mut Context<PlaygroundState>,
) -> AnyElement {
    div().into_any_element()
}
