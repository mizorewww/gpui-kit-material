//! The rules behind the list-detail and supporting pane layouts, as pure
//! functions: which panes show, where they sit and how they move.
//!
//! Source: only m3e lists canonical layouts (the Figma kit and md3.io do not).
//! This is a port of `packages/m3e-motion/src/three-pane-scaffold.ts`, itself a
//! port of androidx `ThreePaneScaffoldValue.kt`, `ThreePaneScaffold.kt`,
//! `PaneMotion.kt`, `PaneExpansionState.kt` and
//! `ThreePaneScaffoldNavigator.kt` at
//! `19991f2c0858aadfcb442ce02b2c8e4743c96898`. Levitated panes and hinges are
//! not ported, as in m3e. Lengths are dp; geometry is in reading order, which
//! the layout mirrors under a right-to-left direction.
//!
//! Copyright 2023 The Android Open Source Project. Licensed under the Apache
//! License, Version 2.0: see `LICENSE-androidx` in this directory.
use super::adaptive::{WindowHeightClass, WindowSizeClass, WindowWidthClass};

/// A pane's role. List-detail: list = secondary, detail = primary. Supporting
/// pane: main = primary, supporting = secondary. Extra is tertiary in both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaneRole {
    /// The highest-priority pane: it takes the width left over.
    Primary,
    /// The second pane to get room.
    Secondary,
    /// The extra pane, last to get room.
    Tertiary,
}

impl PaneRole {
    /// Every role, by priority.
    pub const ALL: [Self; 3] = [Self::Primary, Self::Secondary, Self::Tertiary];

    /// This role's position in [`Self::ALL`].
    pub const fn index(self) -> usize {
        self as usize
    }

    const fn priority(self) -> u8 {
        match self {
            Self::Primary => 10,
            Self::Secondary => 5,
            Self::Tertiary => 1,
        }
    }
}

/// How a pane adapts when there is no partition for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdaptStrategy {
    /// It hides.
    Hide,
    /// It stacks under the given pane, where the height allows.
    Reflow(PaneRole),
}

/// The adapt strategy of each role, in [`PaneRole::ALL`] order.
pub type AdaptStrategies = [AdaptStrategy; 3];

/// List-detail panes all hide.
pub const LIST_DETAIL_STRATEGIES: AdaptStrategies = [AdaptStrategy::Hide; 3];
/// The supporting pane reflows under the main pane; the others hide.
pub const SUPPORTING_PANE_STRATEGIES: AdaptStrategies = [
    AdaptStrategy::Hide,
    AdaptStrategy::Reflow(PaneRole::Primary),
    AdaptStrategy::Hide,
];
/// List-detail's start-to-end order: list, detail, extra.
pub const LIST_DETAIL_ORDER: [PaneRole; 3] =
    [PaneRole::Secondary, PaneRole::Primary, PaneRole::Tertiary];
/// Supporting pane's start-to-end order: main, supporting, extra.
pub const SUPPORTING_PANE_ORDER: [PaneRole; 3] =
    [PaneRole::Primary, PaneRole::Secondary, PaneRole::Tertiary];

/// What a pane is at the moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneAdaptedValue {
    /// In its own partition.
    Expanded,
    /// Stacked under another pane.
    Reflowed,
    /// Not shown.
    Hidden,
}

/// The state of all three panes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreePaneValue(pub [PaneAdaptedValue; 3]);

impl ThreePaneValue {
    /// A role's value.
    pub const fn get(&self, role: PaneRole) -> PaneAdaptedValue {
        self.0[role.index()]
    }
}

/// Which panes get a partition: the latest destination first, then earlier
/// ones, then the rest by priority. With one partition and room for two stacked
/// panes, the first pane that reflows goes under its expanded anchor.
pub fn three_pane_value(
    max_horizontal: usize,
    max_vertical: usize,
    strategies: &AdaptStrategies,
    history: impl DoubleEndedIterator<Item = PaneRole>,
) -> ThreePaneValue {
    let mut expanded = 0;
    let mut values: [Option<PaneAdaptedValue>; 3] = [None; 3];
    let mut check_reflow = max_horizontal == 1
        && max_vertical > 1
        && strategies.iter().any(|s| *s != AdaptStrategy::Hide);
    for pane in history.rev().chain(PaneRole::ALL) {
        let available = expanded < max_horizontal;
        if !available && !check_reflow {
            break;
        }
        if values[pane.index()].is_some() {
            continue;
        }
        let mut reflowed = None;
        let mut anchor = pane;
        let mut anchor_value = None;
        if check_reflow && let AdaptStrategy::Reflow(under) = strategies[pane.index()] {
            reflowed = Some(pane);
            anchor = under;
            anchor_value = values[anchor.index()];
        }
        match anchor_value {
            None => {
                if !available {
                    continue;
                }
                values[anchor.index()] = Some(PaneAdaptedValue::Expanded);
                expanded += 1;
            }
            Some(PaneAdaptedValue::Expanded) => {}
            Some(_) => continue,
        }
        if let Some(pane) = reflowed {
            values[pane.index()] = Some(PaneAdaptedValue::Reflowed);
            check_reflow = false;
        }
    }
    ThreePaneValue(values.map(|value| value.unwrap_or(PaneAdaptedValue::Hidden)))
}

/// How many panes may sit side by side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneRule {
    /// m3.material.io's Breakpoints, through m3e's default: 1 / 1 / 2 / 2 / 3
    /// panes by width class.
    #[default]
    Recommended,
    /// Compose's `calculatePaneScaffoldDirective`: 1 / 1 / 2 / 3 / 3.
    Compose,
}

/// The pane layout a size class calls for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaneDirective {
    /// Panes side by side, at most.
    pub max_horizontal: usize,
    /// The space between side-by-side panes.
    pub horizontal_spacer: f32,
    /// Panes stacked, at most.
    pub max_vertical: usize,
    /// The space between stacked panes.
    pub vertical_spacer: f32,
    /// A pane's default preferred width.
    pub preferred_width: f32,
    /// A pane's default preferred height.
    pub preferred_height: f32,
}

/// The directive for a size class (Compose `PaneScaffoldDirective.kt`).
pub fn pane_directive(size: WindowSizeClass, rule: PaneRule) -> PaneDirective {
    use WindowWidthClass::*;
    let (partitions, horizontal_spacer, preferred_width) = match size.width {
        Compact | Medium => (1, 0., 360.),
        Expanded => (2, 24., 360.),
        Large | ExtraLarge => (3, 24., 412.),
    };
    let stacked = partitions == 1 && size.height == WindowHeightClass::Expanded;
    let recommended = match size.width {
        Compact | Medium => 1,
        Expanded | Large => 2,
        ExtraLarge => 3,
    };
    PaneDirective {
        max_horizontal: match rule {
            PaneRule::Recommended => partitions.min(recommended),
            PaneRule::Compose => partitions,
        },
        horizontal_spacer,
        max_vertical: if stacked { 2 } else { 1 },
        vertical_spacer: if stacked { 24. } else { 0. },
        preferred_width,
        preferred_height: 420.,
    }
}

/// What "back" pops to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackNavigationBehavior {
    /// The previous destination.
    PopLatest,
    /// The latest destination that changes which panes show.
    #[default]
    PopUntilScaffoldValueChange,
    /// The latest destination with another pane.
    PopUntilCurrentDestinationChange,
    /// The latest destination with other content or other panes.
    PopUntilContentChange,
}

/// One entry of a navigator's history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaneDestination<K> {
    /// The pane navigated to.
    pub pane: PaneRole,
    /// What it shows, if the caller keys its content.
    pub content_key: Option<K>,
}

/// A destination history and the pane value it gives. Own it in an `Entity`
/// and hand it to a [`super::PaneScaffold`], which keeps its directive current.
///
/// ```
/// use gpui_kit_material::prelude::*;
///
/// // One pane at a time: the list, then the detail, then back.
/// let mut navigator = PaneNavigator::<u32>::list_detail();
/// assert!(navigator.is_expanded(PaneRole::Secondary));
/// navigator.navigate_to(PaneRole::Primary, Some(7));
/// assert!(navigator.is_expanded(PaneRole::Primary));
/// assert_eq!(navigator.content_key(), Some(&7));
/// navigator.navigate_back(BackNavigationBehavior::default());
/// assert!(navigator.is_expanded(PaneRole::Secondary));
/// ```
#[derive(Clone, Debug)]
pub struct PaneNavigator<K> {
    history: Vec<PaneDestination<K>>,
    directive: PaneDirective,
    strategies: AdaptStrategies,
}

impl<K: Clone + PartialEq> PaneNavigator<K> {
    fn new(first: PaneRole, strategies: AdaptStrategies) -> Self {
        Self {
            history: vec![PaneDestination {
                pane: first,
                content_key: None,
            }],
            directive: pane_directive(
                WindowSizeClass {
                    width: WindowWidthClass::Compact,
                    height: WindowHeightClass::Medium,
                },
                PaneRule::Recommended,
            ),
            strategies,
        }
    }

    /// A list-detail navigator, starting on the list.
    pub fn list_detail() -> Self {
        Self::new(PaneRole::Secondary, LIST_DETAIL_STRATEGIES)
    }

    /// A supporting pane navigator, starting on the main pane.
    pub fn supporting_pane() -> Self {
        Self::new(PaneRole::Primary, SUPPORTING_PANE_STRATEGIES)
    }

    /// The destination history, oldest first.
    pub fn history(&self) -> &[PaneDestination<K>] {
        &self.history
    }

    /// The latest destination.
    pub fn current(&self) -> Option<&PaneDestination<K>> {
        self.history.last()
    }

    /// The latest content key in the history, for what a detail pane shows.
    pub fn content_key(&self) -> Option<&K> {
        self.history
            .iter()
            .rev()
            .find_map(|destination| destination.content_key.as_ref())
    }

    /// The directive in force.
    pub fn directive(&self) -> PaneDirective {
        self.directive
    }

    /// Apply a directive; true when it changed.
    pub fn set_directive(&mut self, directive: PaneDirective) -> bool {
        let changed = self.directive != directive;
        self.directive = directive;
        changed
    }

    /// The adapt strategies in force.
    pub fn strategies(&self) -> &AdaptStrategies {
        &self.strategies
    }

    /// The panes' current value.
    pub fn value(&self) -> ThreePaneValue {
        self.value_at(self.history.len())
    }

    fn value_at(&self, length: usize) -> ThreePaneValue {
        three_pane_value(
            self.directive.max_horizontal,
            self.directive.max_vertical,
            &self.strategies,
            self.history[..length].iter().map(|item| item.pane),
        )
    }

    /// Whether a pane has its own partition, as a list checks before marking
    /// its selected row.
    pub fn is_expanded(&self, pane: PaneRole) -> bool {
        self.value().get(pane) == PaneAdaptedValue::Expanded
    }

    /// Navigate to a pane.
    pub fn navigate_to(&mut self, pane: PaneRole, content_key: Option<K>) {
        self.history.push(PaneDestination { pane, content_key });
    }

    fn previous_index(&self, behavior: BackNavigationBehavior) -> Option<usize> {
        if self.history.len() <= 1 {
            return None;
        }
        let current = self.history.last()?;
        let value = self.value();
        (0..self.history.len() - 1).rev().find(|&index| {
            let item = &self.history[index];
            let value_differs = self.value_at(index + 1) != value;
            match behavior {
                BackNavigationBehavior::PopLatest => true,
                BackNavigationBehavior::PopUntilScaffoldValueChange => value_differs,
                BackNavigationBehavior::PopUntilCurrentDestinationChange => {
                    item.pane != current.pane
                }
                BackNavigationBehavior::PopUntilContentChange => {
                    item.content_key != current.content_key || value_differs
                }
            }
        })
    }

    /// Whether there is a destination to go back to.
    pub fn can_navigate_back(&self, behavior: BackNavigationBehavior) -> bool {
        self.previous_index(behavior).is_some()
    }

    /// Go back; false when there was nothing to go back to, which also clears
    /// the history, as Compose does.
    pub fn navigate_back(&mut self, behavior: BackNavigationBehavior) -> bool {
        match self.previous_index(behavior) {
            Some(index) => {
                self.history.truncate(index + 1);
                true
            }
            None => {
                self.history.clear();
                false
            }
        }
    }
}

/// A pane's bounds in its scaffold.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaneRect {
    /// The distance from the scaffold's left edge.
    pub x: f32,
    /// The distance from the scaffold's top edge.
    pub y: f32,
    /// The width.
    pub width: f32,
    /// The height.
    pub height: f32,
}

impl PaneRect {
    /// The right edge.
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// The rectangle `t` of the way from `self` to `to`.
    pub fn lerp(&self, to: &Self, t: f32) -> Self {
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Self {
            x: mix(self.x, to.x),
            y: mix(self.y, to.y),
            width: mix(self.width, to.width).max(0.),
            height: mix(self.height, to.height).max(0.),
        }
    }
}

/// Everything a pane layout depends on.
#[derive(Clone, Copy, Debug)]
pub struct PaneLayoutInput<'a> {
    /// The scaffold's width.
    pub width: f32,
    /// The scaffold's height.
    pub height: f32,
    /// The directive in force.
    pub directive: PaneDirective,
    /// The panes' value.
    pub value: ThreePaneValue,
    /// The panes from left to right.
    pub order: &'a [PaneRole; 3],
    /// The adapt strategies in force.
    pub strategies: &'a AdaptStrategies,
    /// Whether each role has a pane, in [`PaneRole::ALL`] order.
    pub present: [bool; 3],
    /// Each role's preferred width, where it overrides the directive's.
    pub preferred_widths: [Option<f32>; 3],
    /// The handle's center while it is dragged.
    pub dragging_offset: Option<f32>,
    /// The first pane's width at a resting split.
    pub first_pane_width: Option<f32>,
}

/// The panes' bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaneLayout {
    /// Each role's bounds where it shows, in [`PaneRole::ALL`] order. A pane
    /// collapsed by a split has zero width.
    pub panes: [Option<PaneRect>; 3],
    /// The drag handle's center when exactly two panes are expanded.
    pub handle_offset: Option<f32>,
}

#[derive(Clone, Copy)]
struct Measuring {
    role: PaneRole,
    width: f32,
    height: f32,
    bounds: Option<PaneRect>,
}

/// Up to three panes, so a layout allocates nothing.
#[derive(Clone, Copy)]
struct Panes {
    items: [Measuring; 3],
    len: usize,
}

impl std::ops::Deref for Panes {
    type Target = [Measuring];
    fn deref(&self) -> &[Measuring] {
        &self.items[..self.len]
    }
}

impl std::ops::DerefMut for Panes {
    fn deref_mut(&mut self) -> &mut [Measuring] {
        &mut self.items[..self.len]
    }
}

/// Place the panes (Compose `ThreePaneContentMeasurePolicy`, without hinges).
pub fn three_pane_layout(input: &PaneLayoutInput) -> PaneLayout {
    let (w, h) = (input.width, input.height);
    let directive = input.directive;
    let select = |wanted: PaneAdaptedValue| {
        let empty = Measuring {
            role: PaneRole::Primary,
            width: 0.,
            height: 0.,
            bounds: None,
        };
        let mut panes = Panes {
            items: [empty; 3],
            len: 0,
        };
        for role in input.order {
            if input.present[role.index()] && input.value.get(*role) == wanted {
                panes.items[panes.len] = Measuring {
                    role: *role,
                    width: input.preferred_widths[role.index()]
                        .unwrap_or(directive.preferred_width),
                    height: directive.preferred_height,
                    bounds: None,
                };
                panes.len += 1;
            }
        }
        panes
    };
    let mut expanded = select(PaneAdaptedValue::Expanded);
    let mut reflowed = select(PaneAdaptedValue::Reflowed);
    let spacer = directive.horizontal_spacer;
    let v_spacer = directive.vertical_spacer;
    let outer = PaneRect {
        x: 0.,
        y: 0.,
        width: w,
        height: h,
    };
    let between = |left: f32, right: f32| PaneRect {
        x: left,
        y: 0.,
        width: right - left,
        height: h,
    };
    let in_partition = |partition: PaneRect, pane: &mut Measuring, flowed: &mut Panes| {
        let under = flowed
            .first()
            .and_then(|flowed| match input.strategies[flowed.role.index()] {
                AdaptStrategy::Reflow(anchor) => Some(anchor),
                AdaptStrategy::Hide => None,
            });
        if under == Some(pane.role) {
            let flowed = &mut flowed[0];
            let available = partition.height - v_spacer;
            pane.height = (available - flowed.height).max((available / 2.).trunc());
            flowed.height = available - pane.height;
            flowed.bounds = Some(PaneRect {
                x: partition.x,
                y: partition.y + pane.height + v_spacer,
                width: partition.width,
                height: partition.height - pane.height - v_spacer,
            });
        } else {
            pane.height = partition.height;
        }
        pane.bounds = Some(PaneRect {
            height: pane.height,
            ..partition
        });
    };

    let two = expanded.len() == 2;
    if two && (input.dragging_offset.is_some() || input.first_pane_width.is_some()) {
        let half = (spacer / 2.).trunc();
        let (head, tail) = expanded.split_at_mut(1);
        let (first, second) = (&mut head[0], &mut tail[0]);
        if let Some(offset) = input.dragging_offset {
            if offset <= half {
                let partition = PaneRect {
                    x: offset * 2.,
                    width: w - offset * 2.,
                    ..outer
                };
                in_partition(partition, second, &mut reflowed);
            } else if offset >= w - half {
                let partition = PaneRect {
                    width: offset * 2. - w,
                    ..outer
                };
                in_partition(partition, first, &mut reflowed);
            } else {
                in_partition(between(0., offset - half), first, &mut reflowed);
                in_partition(between(offset + half, w), second, &mut reflowed);
            }
        } else if let Some(resting) = input.first_pane_width {
            let width = resting.clamp(0., w.max(0.));
            if width == 0. {
                in_partition(outer, second, &mut reflowed);
            } else if width >= w - spacer {
                in_partition(outer, first, &mut reflowed);
            } else {
                in_partition(between(0., width), first, &mut reflowed);
                in_partition(between(width + spacer, w), second, &mut reflowed);
            }
        }
    } else if !expanded.is_empty() {
        let allocatable = w - (expanded.len() as f32 - 1.) * spacer;
        let total: f32 = expanded.iter().map(|pane| pane.width).sum();
        if allocatable > total {
            let mut top = 0;
            for (index, pane) in expanded.iter().enumerate() {
                if pane.role.priority() > expanded[top].role.priority() {
                    top = index;
                }
            }
            expanded[top].width += allocatable - total;
        } else if allocatable < total && total > 0. {
            let scale = allocatable.max(0.) / total;
            for pane in expanded.iter_mut() {
                pane.width = (pane.width * scale).trunc();
            }
            // Compose truncates whole pixels and these are fractional dp: what
            // the truncation lost goes to the last pane, or the panes fall short.
            let used: f32 = expanded.iter().map(|pane| pane.width).sum();
            if let Some(last) = expanded.last_mut() {
                last.width += (allocatable.max(0.) - used).max(0.);
            }
        }
        let mut x = 0.;
        for pane in expanded.iter_mut() {
            let partition = PaneRect {
                x,
                y: 0.,
                width: pane.width,
                height: h,
            };
            in_partition(partition, pane, &mut reflowed);
            x += pane.width + spacer;
        }
    }

    let handle_offset = two.then(|| {
        if let Some(offset) = input.dragging_offset {
            offset
        } else {
            match (expanded[0].bounds, expanded[1].bounds) {
                (Some(first), Some(second)) => ((first.right() + second.x) / 2.).trunc(),
                (Some(first), None) => first.right(),
                _ => 0.,
            }
        }
    });
    let mut panes = [None; 3];
    for pane in expanded.iter().chain(reflowed.iter()) {
        panes[pane.role.index()] = Some(pane.bounds.unwrap_or_default());
    }
    PaneLayout {
        panes,
        handle_offset,
    }
}

/// How a pane moves when the value changes (Compose `PaneMotion`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneMotion {
    /// It stays hidden.
    None,
    /// It stays shown, moving to its new bounds.
    AnimateBounds,
    /// It slides in from the left.
    EnterFromLeft,
    /// It slides in from the right.
    EnterFromRight,
    /// It slides in from the left, a little after the others.
    EnterFromLeftDelayed,
    /// It slides in from the right, a little after the others.
    EnterFromRightDelayed,
    /// It slides out to the left.
    ExitToLeft,
    /// It slides out to the right.
    ExitToRight,
    /// It grows from nothing between its neighbours.
    EnterWithExpand,
    /// It shrinks to nothing between its neighbours.
    ExitWithShrink,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MotionType {
    Hidden,
    Exiting,
    Entering,
    Shown,
}

impl PaneMotion {
    fn kind(self) -> MotionType {
        match self {
            Self::None => MotionType::Hidden,
            Self::AnimateBounds => MotionType::Shown,
            Self::EnterFromLeft
            | Self::EnterFromRight
            | Self::EnterFromLeftDelayed
            | Self::EnterFromRightDelayed
            | Self::EnterWithExpand => MotionType::Entering,
            Self::ExitToLeft | Self::ExitToRight | Self::ExitWithShrink => MotionType::Exiting,
        }
    }

    /// Whether this motion starts after the others (Compose's delayed spring).
    pub const fn delayed(self) -> bool {
        matches!(
            self,
            Self::EnterFromLeftDelayed | Self::EnterFromRightDelayed
        )
    }
}

/// The motion of each role, in [`PaneRole::ALL`] order, for a change of value.
pub fn pane_motions(
    previous: ThreePaneValue,
    current: ThreePaneValue,
    order: &[PaneRole; 3],
) -> [PaneMotion; 3] {
    const N: isize = 3;
    let shows = |value: PaneAdaptedValue| value != PaneAdaptedValue::Hidden;
    let types = order.map(
        |role| match (shows(previous.get(role)), shows(current.get(role))) {
            (true, true) => MotionType::Shown,
            (true, false) => MotionType::Exiting,
            (false, true) => MotionType::Entering,
            (false, false) => MotionType::Hidden,
        },
    );
    let mut motions = [PaneMotion::None; 3];
    let (mut first_shown, mut first_entering) = (N, N);
    let (mut last_shown, mut last_entering) = (-1, -1);
    for (i, kind) in types.iter().enumerate() {
        let i = i as isize;
        match kind {
            MotionType::Shown => {
                first_shown = first_shown.min(i);
                last_shown = last_shown.max(i);
                motions[i as usize] = PaneMotion::AnimateBounds;
            }
            MotionType::Entering => {
                first_entering = first_entering.min(i);
                last_entering = last_entering.max(i);
            }
            _ => {}
        }
    }
    let (mut exit_right, mut exit_left) = (false, false);
    let (mut first_exit_right, mut last_exit_left) = (N, -1);
    for (index, kind) in types.iter().enumerate() {
        if *kind != MotionType::Exiting {
            continue;
        }
        let i = index as isize;
        let (shown_left, entering_left) = (first_shown < i, first_entering < i);
        let (shown_right, entering_right) = (last_shown > i, last_entering > i);
        let to_right =
            (!shown_right && !entering_right) || ((shown_left || entering_left) && !shown_right);
        let to_left = !to_right && ((!shown_left && !entering_left) || !shown_left);
        if to_right {
            exit_right = true;
            first_exit_right = first_exit_right.min(i);
            motions[index] = PaneMotion::ExitToRight;
        } else if to_left {
            exit_left = true;
            last_exit_left = last_exit_left.max(i);
            motions[index] = PaneMotion::ExitToLeft;
        } else {
            motions[index] = PaneMotion::ExitWithShrink;
        }
    }
    for (index, kind) in types.iter().enumerate() {
        if *kind != MotionType::Entering {
            continue;
        }
        let i = index as isize;
        let free_right = last_shown <= i && last_exit_left <= i;
        let free_left = first_shown >= i && first_exit_right >= i;
        motions[index] = if free_right && !exit_right {
            PaneMotion::EnterFromRight
        } else if free_left && !exit_left {
            PaneMotion::EnterFromLeft
        } else if free_right {
            PaneMotion::EnterFromRightDelayed
        } else if free_left {
            PaneMotion::EnterFromLeftDelayed
        } else {
            PaneMotion::EnterWithExpand
        };
    }
    let mut out = [PaneMotion::None; 3];
    for (index, role) in order.iter().enumerate() {
        out[role.index()] = motions[index];
    }
    out
}

/// Where a pane moves from and to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaneTransition {
    /// The motion this realizes.
    pub motion: PaneMotion,
    /// The bounds it starts at.
    pub from: PaneRect,
    /// The bounds it ends at.
    pub to: PaneRect,
}

/// The transition of each role, in [`PaneRole::ALL`] order. `origin` holds the
/// bounds before the change and `target` those after it.
pub fn pane_transitions(
    motions: &[PaneMotion; 3],
    order: &[PaneRole; 3],
    origin: &[Option<PaneRect>; 3],
    target: &[Option<PaneRect>; 3],
    scaffold_width: f32,
) -> [Option<PaneTransition>; 3] {
    let t = |r: PaneRole| target[r.index()].or(origin[r.index()]).unwrap_or_default();
    let o = |r: PaneRole| origin[r.index()].or(target[r.index()]).unwrap_or_default();
    let kind = |r: PaneRole| motions[r.index()].kind();
    let shown = |r: PaneRole, other: MotionType| kind(r) == MotionType::Shown || kind(r) == other;

    let slide_in_left = || {
        let mut previous = None;
        for r in order.iter().rev().copied() {
            if matches!(
                motions[r.index()],
                PaneMotion::EnterFromLeft | PaneMotion::EnterFromLeftDelayed
            ) {
                return -previous.map_or(t(r).right(), |p: PaneRole| t(p).x);
            }
            if shown(r, MotionType::Entering) {
                previous = Some(r);
            }
        }
        0.
    };
    let slide_in_right = || {
        let mut previous = None;
        for r in order.iter().copied() {
            if matches!(
                motions[r.index()],
                PaneMotion::EnterFromRight | PaneMotion::EnterFromRightDelayed
            ) {
                return scaffold_width - previous.map_or(t(r).x, |p: PaneRole| t(p).right());
            }
            if shown(r, MotionType::Entering) {
                previous = Some(r);
            }
        }
        0.
    };
    let slide_out_left = || {
        let mut previous = None;
        for r in order.iter().rev().copied() {
            if motions[r.index()] == PaneMotion::ExitToLeft {
                return -previous.map_or(o(r).right(), |p: PaneRole| o(p).x);
            }
            if shown(r, MotionType::Exiting) {
                previous = Some(r);
            }
        }
        0.
    };
    let slide_out_right = || {
        let mut previous = None;
        for r in order.iter().copied() {
            if motions[r.index()] == PaneMotion::ExitToRight {
                return scaffold_width - previous.map_or(o(r).x, |p: PaneRole| o(p).right());
            }
            if shown(r, MotionType::Exiting) {
                previous = Some(r);
            }
        }
        0.
    };
    let left_edge = |role: PaneRole, other: MotionType, rect: &dyn Fn(PaneRole) -> PaneRect| {
        let mut edge = 0.;
        for r in order.iter().copied() {
            if r == role {
                return edge;
            }
            if shown(r, other) {
                edge = rect(r).right();
            }
        }
        edge
    };
    let shift = |rect: PaneRect, dx: f32| PaneRect {
        x: rect.x + dx,
        ..rect
    };

    let mut out = [None; 3];
    for r in order.iter().copied() {
        let motion = motions[r.index()];
        let (to, from) = (t(r), o(r));
        out[r.index()] = match motion {
            PaneMotion::None => None,
            PaneMotion::AnimateBounds => Some(PaneTransition { motion, from, to }),
            PaneMotion::EnterFromLeft | PaneMotion::EnterFromLeftDelayed => Some(PaneTransition {
                motion,
                from: shift(to, slide_in_left()),
                to,
            }),
            PaneMotion::EnterFromRight | PaneMotion::EnterFromRightDelayed => {
                Some(PaneTransition {
                    motion,
                    from: shift(to, slide_in_right()),
                    to,
                })
            }
            PaneMotion::EnterWithExpand => Some(PaneTransition {
                motion,
                from: PaneRect {
                    x: left_edge(r, MotionType::Exiting, &o),
                    width: 0.,
                    ..to
                },
                to,
            }),
            PaneMotion::ExitToLeft => Some(PaneTransition {
                motion,
                from,
                to: shift(from, slide_out_left()),
            }),
            PaneMotion::ExitToRight => Some(PaneTransition {
                motion,
                from,
                to: shift(from, slide_out_right()),
            }),
            PaneMotion::ExitWithShrink => Some(PaneTransition {
                motion,
                from,
                to: PaneRect {
                    x: left_edge(r, MotionType::Entering, &t),
                    width: 0.,
                    ..from
                },
            }),
        };
    }
    out
}

/// A position the drag handle settles at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaneAnchor {
    /// A fraction of the scaffold's width.
    Proportion(f32),
    /// Dp from the left edge.
    FromStart(f32),
    /// Dp from the right edge.
    FromEnd(f32),
}

impl PaneAnchor {
    /// The handle's center for this anchor in a scaffold `width` wide.
    pub fn position(self, width: f32) -> f32 {
        match self {
            Self::Proportion(fraction) => (width * fraction).round().clamp(0., width.max(0.)),
            Self::FromStart(offset) => offset.round().clamp(0., width.max(0.)),
            Self::FromEnd(offset) => (width - offset.round()).clamp(0., width.max(0.)),
        }
    }
}

/// The Compose sample's anchors, which m3e uses by default: 0%, 240dp from the
/// start, 50%, 240dp from the end, 100%.
pub const DEFAULT_PANE_ANCHORS: [PaneAnchor; 5] = [
    PaneAnchor::Proportion(0.),
    PaneAnchor::FromStart(240.),
    PaneAnchor::Proportion(0.5),
    PaneAnchor::FromEnd(240.),
    PaneAnchor::Proportion(1.),
];

/// The anchor position closest to `position`.
pub fn closest_pane_anchor(anchors: &[PaneAnchor], width: f32, position: f32) -> Option<f32> {
    anchors
        .iter()
        .map(|anchor| anchor.position(width))
        .fold(None, |best: Option<f32>, at| match best {
            Some(best) if (best - position).abs() <= (at - position).abs() => Some(best),
            _ => Some(at),
        })
}

/// The first anchor position right of `position`, wrapping to the leftmost.
pub fn next_pane_anchor(anchors: &[PaneAnchor], width: f32, position: f32) -> Option<f32> {
    let mut sorted: Vec<f32> = anchors
        .iter()
        .map(|anchor| anchor.position(width))
        .collect();
    sorted.sort_by(f32::total_cmp);
    sorted
        .iter()
        .find(|at| position < **at)
        .or(sorted.first())
        .copied()
}

/// The first anchor position left of `position`, wrapping to the rightmost.
pub fn previous_pane_anchor(anchors: &[PaneAnchor], width: f32, position: f32) -> Option<f32> {
    let mut sorted: Vec<f32> = anchors
        .iter()
        .map(|anchor| anchor.position(width))
        .collect();
    sorted.sort_by(f32::total_cmp);
    sorted
        .iter()
        .rev()
        .find(|at| position > **at)
        .or(sorted.last())
        .copied()
}

#[cfg(test)]
mod tests {
    use super::PaneAdaptedValue::{Expanded, Hidden, Reflowed};
    use super::PaneRole::{Primary, Secondary, Tertiary};
    use super::*;
    use core::prelude::v1::test;

    fn class(width: f32, height: f32) -> WindowSizeClass {
        WindowSizeClass::of(width, height)
    }

    fn value(
        horizontal: usize,
        vertical: usize,
        strategies: &AdaptStrategies,
        history: &[PaneRole],
    ) -> [PaneAdaptedValue; 3] {
        three_pane_value(horizontal, vertical, strategies, history.iter().copied()).0
    }

    #[test]
    fn pane_counts_follow_the_rule_at_each_width() {
        for (width, recommended, compose) in [
            (360., 1, 1),
            (700., 1, 1),
            (1000., 2, 2),
            (1300., 2, 3),
            (1700., 3, 3),
        ] {
            let size = class(width, 800.);
            assert_eq!(
                pane_directive(size, PaneRule::Recommended).max_horizontal,
                recommended,
                "{width}"
            );
            assert_eq!(
                pane_directive(size, PaneRule::Compose).max_horizontal,
                compose,
                "{width}"
            );
        }
    }

    #[test]
    fn directive_values_are_composes() {
        let compact = pane_directive(class(360., 800.), PaneRule::Recommended);
        assert_eq!(compact.horizontal_spacer, 0.);
        assert_eq!(compact.preferred_width, 360.);
        assert_eq!(compact.max_vertical, 1);
        let tall = pane_directive(class(360., 900.), PaneRule::Recommended);
        assert_eq!((tall.max_vertical, tall.vertical_spacer), (2, 24.));
        let expanded = pane_directive(class(1000., 900.), PaneRule::Recommended);
        assert_eq!(expanded.horizontal_spacer, 24.);
        assert_eq!(expanded.max_vertical, 1, "stacking is for a single pane");
        let large = pane_directive(class(1300., 800.), PaneRule::Recommended);
        assert_eq!(large.preferred_width, 412.);
        assert_eq!(large.preferred_height, 420.);
    }

    #[test]
    fn the_latest_destination_takes_the_only_partition() {
        let list = [Secondary];
        assert_eq!(
            value(1, 1, &LIST_DETAIL_STRATEGIES, &list),
            [Hidden, Expanded, Hidden]
        );
        assert_eq!(
            value(1, 1, &LIST_DETAIL_STRATEGIES, &[Secondary, Primary]),
            [Expanded, Hidden, Hidden]
        );
    }

    #[test]
    fn a_second_partition_goes_to_the_next_in_the_history_then_by_priority() {
        assert_eq!(
            value(2, 1, &LIST_DETAIL_STRATEGIES, &[Secondary]),
            [Expanded, Expanded, Hidden]
        );
        assert_eq!(
            value(
                2,
                1,
                &LIST_DETAIL_STRATEGIES,
                &[Secondary, Primary, Tertiary]
            ),
            [Expanded, Hidden, Expanded]
        );
        assert_eq!(
            value(3, 1, &LIST_DETAIL_STRATEGIES, &[Secondary]),
            [Expanded, Expanded, Expanded]
        );
    }

    #[test]
    fn the_supporting_pane_reflows_under_the_main_pane_when_tall() {
        assert_eq!(
            value(1, 2, &SUPPORTING_PANE_STRATEGIES, &[Primary]),
            [Expanded, Reflowed, Hidden]
        );
        assert_eq!(
            value(1, 1, &SUPPORTING_PANE_STRATEGIES, &[Primary]),
            [Expanded, Hidden, Hidden]
        );
    }

    #[test]
    fn back_pops_until_the_panes_change() {
        let mut navigator = PaneNavigator::<u32>::list_detail();
        navigator.set_directive(pane_directive(class(360., 800.), PaneRule::Recommended));
        assert!(!navigator.can_navigate_back(BackNavigationBehavior::default()));
        navigator.navigate_to(Primary, Some(1));
        navigator.navigate_to(Primary, Some(2));
        assert_eq!(navigator.content_key(), Some(&2));
        assert!(navigator.can_navigate_back(BackNavigationBehavior::PopLatest));
        // Two details look the same: the default skips to the list.
        assert!(navigator.navigate_back(BackNavigationBehavior::default()));
        assert_eq!(navigator.current().unwrap().pane, Secondary);
        assert_eq!(navigator.history().len(), 1);
        // Nothing left: it clears, and the value falls back to priority order.
        assert!(!navigator.navigate_back(BackNavigationBehavior::default()));
        assert!(navigator.history().is_empty());
        assert_eq!(navigator.value().get(Primary), Expanded);
    }

    #[test]
    fn back_behaviors_differ_as_composes_do() {
        let mut navigator = PaneNavigator::<u32>::list_detail();
        navigator.set_directive(pane_directive(class(1000., 800.), PaneRule::Recommended));
        navigator.navigate_to(Primary, Some(1));
        navigator.navigate_to(Primary, Some(2));
        // Both panes show either way, so no scaffold change is available...
        assert!(!navigator.can_navigate_back(BackNavigationBehavior::PopUntilScaffoldValueChange));
        // ...though the content does change, and the pane does further back.
        let mut content = navigator.clone();
        assert!(content.navigate_back(BackNavigationBehavior::PopUntilContentChange));
        assert_eq!(content.content_key(), Some(&1));
        let mut pane = navigator.clone();
        assert!(pane.navigate_back(BackNavigationBehavior::PopUntilCurrentDestinationChange));
        assert_eq!(pane.current().unwrap().pane, Secondary);
    }

    fn layout(width: f32, height: f32, history: &[PaneRole]) -> PaneLayout {
        let directive = pane_directive(class(width, height), PaneRule::Recommended);
        three_pane_layout(&PaneLayoutInput {
            width,
            height,
            directive,
            value: three_pane_value(
                directive.max_horizontal,
                directive.max_vertical,
                &LIST_DETAIL_STRATEGIES,
                history.iter().copied(),
            ),
            order: &LIST_DETAIL_ORDER,
            strategies: &LIST_DETAIL_STRATEGIES,
            present: [true; 3],
            preferred_widths: [None; 3],
            dragging_offset: None,
            first_pane_width: None,
        })
    }

    #[test]
    fn the_primary_pane_takes_what_the_others_leave() {
        let two = layout(1000., 800., &[Secondary]);
        let list = two.panes[Secondary.index()].unwrap();
        let detail = two.panes[Primary.index()].unwrap();
        assert_eq!((list.x, list.width), (0., 360.));
        assert_eq!((detail.x, detail.width), (384., 616.));
        assert_eq!(two.handle_offset, Some(372.));
        assert!(two.panes[Tertiary.index()].is_none());
    }

    #[test]
    fn panes_that_do_not_fit_shrink_by_the_same_ratio() {
        let directive = PaneDirective {
            max_horizontal: 2,
            ..pane_directive(class(1000., 800.), PaneRule::Recommended)
        };
        let result = three_pane_layout(&PaneLayoutInput {
            width: 500.,
            height: 800.,
            directive,
            value: ThreePaneValue([Expanded, Expanded, Hidden]),
            order: &LIST_DETAIL_ORDER,
            strategies: &LIST_DETAIL_STRATEGIES,
            present: [true; 3],
            preferred_widths: [None; 3],
            dragging_offset: None,
            first_pane_width: None,
        });
        let (first, second) = (
            result.panes[Secondary.index()].unwrap(),
            result.panes[Primary.index()].unwrap(),
        );
        assert_eq!(first.width, second.width);
        assert!(first.width + second.width + 24. <= 500.);
    }

    #[test]
    fn a_reflowed_pane_sits_under_its_anchor_at_its_preferred_height() {
        let directive = pane_directive(class(700., 1000.), PaneRule::Recommended);
        let result = three_pane_layout(&PaneLayoutInput {
            width: 700.,
            height: 1000.,
            directive,
            value: ThreePaneValue([Expanded, Reflowed, Hidden]),
            order: &SUPPORTING_PANE_ORDER,
            strategies: &SUPPORTING_PANE_STRATEGIES,
            present: [true; 3],
            preferred_widths: [None; 3],
            dragging_offset: None,
            first_pane_width: None,
        });
        let main = result.panes[Primary.index()].unwrap();
        let supporting = result.panes[Secondary.index()].unwrap();
        assert_eq!(main.width, 700.);
        assert_eq!(main.height, 1000. - 24. - 420.);
        assert_eq!(supporting.y, main.height + 24.);
        assert_eq!(supporting.height, 420.);
        assert_eq!(result.handle_offset, None);
    }

    #[test]
    fn a_split_at_an_edge_collapses_one_pane() {
        let directive = pane_directive(class(1000., 800.), PaneRule::Recommended);
        let split = |first_pane_width| {
            three_pane_layout(&PaneLayoutInput {
                width: 1000.,
                height: 800.,
                directive,
                value: ThreePaneValue([Expanded, Expanded, Hidden]),
                order: &LIST_DETAIL_ORDER,
                strategies: &LIST_DETAIL_STRATEGIES,
                present: [true; 3],
                preferred_widths: [None; 3],
                dragging_offset: None,
                first_pane_width: Some(first_pane_width),
            })
        };
        let all_second = split(0.);
        assert_eq!(all_second.panes[Primary.index()].unwrap().width, 1000.);
        assert_eq!(all_second.panes[Secondary.index()].unwrap().width, 0.);
        assert_eq!(all_second.handle_offset, Some(0.));
        let all_first = split(1000.);
        assert_eq!(all_first.panes[Secondary.index()].unwrap().width, 1000.);
        let middle = split(500.);
        assert_eq!(middle.panes[Secondary.index()].unwrap().width, 500.);
        assert_eq!(middle.panes[Primary.index()].unwrap().x, 524.);
    }

    #[test]
    fn extreme_sizes_do_not_panic() {
        for (width, height) in [(0., 0.), (1., 1.), (20000., 20000.)] {
            let result = layout(width, height, &[Secondary]);
            assert!(result.panes.iter().flatten().all(|pane| pane.width >= 0.));
        }
    }

    #[test]
    fn navigating_in_one_pane_slides_the_new_pane_over_the_old() {
        let before = ThreePaneValue([Hidden, Expanded, Hidden]);
        let after = ThreePaneValue([Expanded, Hidden, Hidden]);
        // The list is left of the detail: forward, the list leaves left and
        // the detail enters from the right.
        let forward = pane_motions(before, after, &LIST_DETAIL_ORDER);
        assert_eq!(forward[Secondary.index()], PaneMotion::ExitToLeft);
        assert_eq!(forward[Primary.index()], PaneMotion::EnterFromRight);
        assert_eq!(forward[Tertiary.index()], PaneMotion::None);
        // Back reverses it.
        let back = pane_motions(after, before, &LIST_DETAIL_ORDER);
        assert_eq!(back[Primary.index()], PaneMotion::ExitToRight);
        assert_eq!(back[Secondary.index()], PaneMotion::EnterFromLeft);
    }

    #[test]
    fn a_pane_stays_shown_and_animates_its_bounds() {
        let before = ThreePaneValue([Expanded, Expanded, Hidden]);
        let after = ThreePaneValue([Expanded, Hidden, Expanded]);
        let motions = pane_motions(before, after, &LIST_DETAIL_ORDER);
        assert_eq!(motions[Primary.index()], PaneMotion::AnimateBounds);
        assert_eq!(motions[Secondary.index()], PaneMotion::ExitToLeft);
        assert_eq!(motions[Tertiary.index()], PaneMotion::EnterFromRight);
    }

    #[test]
    fn transitions_start_and_end_outside_the_scaffold() {
        let order = LIST_DETAIL_ORDER;
        let list = PaneRect {
            x: 0.,
            y: 0.,
            width: 400.,
            height: 800.,
        };
        let motions = pane_motions(
            ThreePaneValue([Hidden, Expanded, Hidden]),
            ThreePaneValue([Expanded, Hidden, Hidden]),
            &order,
        );
        let origin = [None, Some(list), None];
        let target = [Some(list), None, None];
        let result = pane_transitions(&motions, &order, &origin, &target, 400.);
        let leaving = result[Secondary.index()].unwrap();
        assert_eq!(leaving.from, list);
        assert_eq!(leaving.to.x, -400.);
        let entering = result[Primary.index()].unwrap();
        assert_eq!(entering.to, list);
        assert_eq!(entering.from.x, 400.);
        assert!(result[Tertiary.index()].is_none());
    }

    #[test]
    fn anchors_resolve_and_wrap() {
        let anchors = DEFAULT_PANE_ANCHORS;
        assert_eq!(anchors[0].position(1000.), 0.);
        assert_eq!(anchors[1].position(1000.), 240.);
        assert_eq!(anchors[2].position(1000.), 500.);
        assert_eq!(anchors[3].position(1000.), 760.);
        assert_eq!(anchors[4].position(1000.), 1000.);
        assert_eq!(closest_pane_anchor(&anchors, 1000., 300.), Some(240.));
        assert_eq!(next_pane_anchor(&anchors, 1000., 500.), Some(760.));
        assert_eq!(next_pane_anchor(&anchors, 1000., 1000.), Some(0.));
        assert_eq!(previous_pane_anchor(&anchors, 1000., 500.), Some(240.));
        assert_eq!(previous_pane_anchor(&anchors, 1000., 0.), Some(1000.));
        assert_eq!(closest_pane_anchor(&[], 1000., 300.), None);
    }

    #[test]
    fn a_shrunk_pane_still_fills_the_scaffold() {
        for width in [113., 181., 226., 250.] {
            let panes = layout(width, 800., &[PaneRole::Secondary]);
            let total: f32 = panes.panes.iter().flatten().map(|pane| pane.width).sum();
            assert_eq!(total, width, "{panes:?}");
        }
    }

    #[test]
    fn offset_anchors_stay_inside_the_scaffold() {
        assert_eq!(PaneAnchor::FromStart(240.).position(200.), 200.);
        assert_eq!(PaneAnchor::FromEnd(240.).position(200.), 0.);
        assert_eq!(
            next_pane_anchor(&DEFAULT_PANE_ANCHORS, 200., 200.),
            Some(0.),
            "the handle wraps rather than sticks"
        );
    }
}
