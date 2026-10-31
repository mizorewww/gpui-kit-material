//! The page's address in the browser: `#button`, `#button/specs`. A link opens
//! that page, and the Back gesture, which a phone user reaches for first,
//! returns to the page before instead of leaving the Gallery.
use super::Gallery;
use super::catalog::{parse_route, route_hash};
use gpui::{prelude::*, *};
use wasm_bindgen::{JsCast, closure::Closure};

fn current_hash() -> String {
    web_sys::window()
        .and_then(|window| window.location().hash().ok())
        .unwrap_or_default()
}

fn apply(gallery: &mut Gallery, hash: &str, cx: &mut Context<Gallery>) {
    let (entry, tab) = if hash.is_empty() {
        (0, None)
    } else {
        match parse_route(hash) {
            Some(route) => route,
            None => return,
        }
    };
    gallery.navigate(entry, cx);
    if let Some(tab) = tab {
        gallery.set_tab(tab, cx);
    }
}

/// Opens the address's page, then keeps the two in step for the page's life.
pub(super) fn install(gallery: &Entity<Gallery>, cx: &mut App) {
    gallery.update(cx, |gallery, cx| apply(gallery, &current_hash(), cx));

    let mut shown = {
        let gallery = gallery.read(cx);
        (gallery.current, gallery.entries[gallery.current].tab)
    };
    cx.observe(gallery, move |gallery, cx| {
        let gallery = gallery.read(cx);
        let now = (gallery.current, gallery.entries[gallery.current].tab);
        if now != shown {
            shown = now;
            if let Some(window) = web_sys::window() {
                // Adds a history entry; the event it fires finds nothing to do.
                let _ = window.location().set_hash(&route_hash(now.0, now.1));
            }
        }
    })
    .detach();

    let weak = gallery.downgrade();
    let app = cx.to_async();
    let listener = Closure::<dyn FnMut()>::new(move || {
        let hash = current_hash();
        app.update(|cx| {
            let _ = weak.update(cx, |gallery, cx| apply(gallery, &hash, cx));
        });
    });
    if let Some(window) = web_sys::window() {
        let _ = window
            .add_event_listener_with_callback("hashchange", listener.as_ref().unchecked_ref());
    }
    // The Gallery lives as long as the page.
    listener.forget();
}
