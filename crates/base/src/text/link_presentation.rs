use std::{
    cell::RefCell,
    ops::Range,
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

use gpui::{
    HighlightStyle, Hitbox, MouseButton, MouseDownEvent, Pixels, Point, TextLayout, Window,
};

use super::{inline::InlineState, node::LinkMark};

/// Metric-preserving link paint supplied by a caller. No history or opening capability.
#[derive(Clone)]
pub struct LinkPresentation {
    pub normal: HighlightStyle,
    pub hover: HighlightStyle,
    pub pressed: HighlightStyle,
    pub visited: HighlightStyle,
    pub revision: u64,
    pub is_visited: Arc<dyn Fn(&str) -> bool + Send + Sync>,
}

impl LinkPresentation {
    pub(super) fn appearance(&self, hovered: bool, pressed: bool, url: &str) -> HighlightStyle {
        let appearance = if pressed {
            self.pressed
        } else if hovered {
            self.hover
        } else if (self.is_visited)(url) {
            self.visited
        } else {
            self.normal
        };
        HighlightStyle {
            color: appearance.color,
            background_color: appearance.background_color,
            underline: appearance.underline,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub(super) struct LinkOccurrence {
    leaf: Weak<Mutex<InlineState>>,
    range: Range<usize>,
}

impl PartialEq for LinkOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.leaf.ptr_eq(&other.leaf) && self.range == other.range
    }
}

impl LinkOccurrence {
    pub(super) fn new(leaf: &Arc<Mutex<InlineState>>, range: Range<usize>) -> Self {
        Self {
            leaf: Arc::downgrade(leaf),
            range,
        }
    }
}

pub(super) struct LinkTarget {
    pub occurrence: LinkOccurrence,
    pub range: Range<usize>,
    pub link: LinkMark,
    pub layout: TextLayout,
    pub hitbox: Hitbox,
}

#[derive(Default)]
pub(super) struct LinkInteraction {
    pub gesture: super::link_gesture::LinkGesture<LinkOccurrence, MouseDownEvent>,
    pub targets: Vec<LinkTarget>,
}

pub(super) type SharedLinkInteraction = Rc<RefCell<LinkInteraction>>;

impl LinkInteraction {
    pub fn target(&self, position: Point<Pixels>, window: &Window) -> Option<&LinkTarget> {
        if !window.is_window_active() {
            return None;
        }
        self.targets.iter().rev().find(|target| {
            target.hitbox.is_hovered(window)
                && target
                    .layout
                    .index_for_position(position)
                    .ok()
                    .is_some_and(|offset| target.range.contains(&offset))
        })
    }

    pub fn reconcile(&mut self, position: Point<Pixels>, window: &Window, dragging: bool) -> bool {
        let hovered = self
            .target(position, window)
            .map(|target| target.occurrence.clone());
        let cancelled = dragging
            || !window.is_window_active()
            || self
                .gesture
                .pressed
                .as_ref()
                .is_some_and(|(_, down)| (position - down.position).magnitude() > 3.);
        self.gesture.reconcile(hovered, cancelled)
    }

    pub fn down(&mut self, event: &MouseDownEvent, window: &Window) -> bool {
        let was_pressed = self.gesture.pressed.is_some();
        let target = (event.click_count == 1
            && matches!(event.button, MouseButton::Left | MouseButton::Middle))
        .then(|| {
            self.target(event.position, window)
                .map(|target| target.occurrence.clone())
        })
        .flatten();
        self.gesture.down(target, event.clone()) || was_pressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_prioritizes_pressed_hover_and_visited_without_changing_metrics() {
        let style = |color| HighlightStyle {
            color: Some(gpui::rgb(color).into()),
            font_weight: Some(gpui::FontWeight::BOLD),
            font_style: Some(gpui::FontStyle::Italic),
            ..Default::default()
        };
        let presentation = LinkPresentation {
            normal: style(0x111111),
            hover: style(0x222222),
            pressed: style(0x333333),
            visited: style(0x444444),
            revision: 0,
            is_visited: Arc::new(|url| url == "visited"),
        };
        for (hover, pressed, url, expected) in [
            (false, false, "new", presentation.normal),
            (false, false, "visited", presentation.visited),
            (true, false, "visited", presentation.hover),
            (true, true, "visited", presentation.pressed),
            (false, true, "new", presentation.pressed),
        ] {
            let appearance = presentation.appearance(hover, pressed, url);
            assert_eq!(appearance.color, expected.color);
            assert_eq!(appearance.font_weight, None);
            assert_eq!(appearance.font_style, None);
        }
    }
}
