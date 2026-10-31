// Modified by Longbridge for gpui-fast.
//! Tests of how retained subtrees record what they read. See
//! [`crate::fast::dependencies`].

use crate::fast::dependencies::merge_sorted;
use std::rc::Rc;

#[test]
fn merging_sorted_lists_shares_one_that_holds_the_other() {
    let all: Rc<[u32]> = Rc::from([1, 3, 5, 7]);
    let some: Rc<[u32]> = Rc::from([3, 7]);
    let empty: Rc<[u32]> = Rc::from([]);

    assert!(Rc::ptr_eq(&merge_sorted(&all, &some), &all));
    assert!(Rc::ptr_eq(&merge_sorted(&some, &all), &all));
    assert!(Rc::ptr_eq(&merge_sorted(&all, &empty), &all));
    assert!(Rc::ptr_eq(&merge_sorted(&empty, &some), &some));

    let other: Rc<[u32]> = Rc::from([2, 3, 8]);
    assert_eq!(&*merge_sorted(&all, &other), &[1, 2, 3, 5, 7, 8]);
    assert_eq!(&*merge_sorted(&other, &all), &[1, 2, 3, 5, 7, 8]);
}

/// A subtree built again, frame after frame, reading what it read before,
/// takes the list of what it read that it had, rather than a new one.
#[test]
fn a_subtree_built_again_reading_the_same_takes_the_list_it_had() {
    use crate::{
        AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, TestAppContext,
        Window, div,
    };

    struct Shared(u32);
    struct Reader(Entity<Shared>);
    impl Render for Reader {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().child(format!("{}", self.0.read(cx).0))
        }
    }
    struct Readers(Vec<Entity<Reader>>);
    impl Render for Readers {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().children(self.0.iter().cloned())
        }
    }

    let mut cx = TestAppContext::single();
    let window = cx.add_window(|_, cx| {
        let shared = cx.new(|_| Shared(0));
        Readers((0..3).map(|_| cx.new(|_| Reader(shared.clone()))).collect())
    });
    let lists = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            // Every reader built again, reading what it read before.
            window.refresh();
            window.draw(cx).clear(cx);
            window
                .rendered_frame
                .retained
                .records
                .iter()
                .filter(|record| record.own_dependencies.entities.len() == 2)
                .map(|record| record.own_dependencies.entities.clone())
                .collect::<Vec<_>>()
        })
        .unwrap()
    };
    let first = lists(&mut cx);
    assert_eq!(first.len(), 3, "each reader reads itself and the model");
    let second = lists(&mut cx);
    assert_eq!(second.len(), 3);
    for (first, second) in first.iter().zip(&second) {
        assert!(
            Rc::ptr_eq(first, second),
            "a subtree built again reading the same should take the list it had"
        );
    }
}

/// A lease to build from an entity leaves what read it alone; an update, or a
/// notify during the lease, builds it again.
#[cfg(feature = "frame-observation")]
#[test]
fn a_render_lease_is_not_a_change_unless_it_notifies() {
    use crate::{
        AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, TestAppContext,
        Window, div,
    };
    use std::cell::Cell;

    struct Shared(u32);
    struct Reader {
        shared: Entity<Shared>,
        renders: Rc<Cell<usize>>,
    }
    impl Render for Reader {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.renders.set(self.renders.get() + 1);
            div().child(format!("{}", self.shared.read(cx).0))
        }
    }
    struct Root(Entity<Reader>);
    impl Render for Root {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(self.0.clone())
        }
    }

    let mut cx = TestAppContext::single();
    let renders = Rc::new(Cell::new(0));
    let shared = cx.new(|_| Shared(0));
    let window = cx.add_window({
        let (renders, shared) = (renders.clone(), shared.clone());
        move |_, cx| Root(cx.new(|_| Reader { shared, renders }))
    });
    let mut draw_after = |change: &dyn Fn(&mut crate::App)| {
        cx.update_window(window.into(), |root, window, cx| {
            window.draw(cx).clear(cx);
            let before = renders.get();
            change(cx);
            cx.notify(root.entity_id());
            window.draw(cx).clear(cx);
            renders.get() - before
        })
        .unwrap()
    };
    assert_eq!(draw_after(&|cx| shared.update_for_render(cx, |_, _| {})), 0);
    assert_eq!(draw_after(&|cx| shared.update(cx, |_, _| {})), 1);
    assert_eq!(
        draw_after(&|cx| shared.update_for_render(cx, |_, cx| cx.notify())),
        1
    );
}
