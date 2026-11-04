pub struct LinkGesture<T, D> {
    pub hovered: Option<T>,
    pub pressed: Option<(T, D)>,
}

impl<T, D> Default for LinkGesture<T, D> {
    fn default() -> Self {
        Self {
            hovered: None,
            pressed: None,
        }
    }
}

impl<T: Clone + PartialEq, D> LinkGesture<T, D> {
    pub fn reconcile(&mut self, hovered: Option<T>, cancelled: bool) -> bool {
        let changed = self.hovered != hovered;
        self.hovered = hovered;
        let clear = self
            .pressed
            .as_ref()
            .is_some_and(|(occurrence, _)| cancelled || self.hovered.as_ref() != Some(occurrence));
        if clear {
            self.pressed = None;
        }
        changed || clear
    }

    pub fn down(&mut self, target: Option<T>, down: D) -> bool {
        self.pressed = target.map(|target| (target, down));
        self.pressed.is_some()
    }

    pub fn release(&mut self, target: Option<&T>) -> Option<D> {
        self.pressed
            .take()
            .and_then(|(occurrence, down)| (target == Some(&occurrence)).then_some(down))
    }
}

#[cfg(test)]
mod tests {
    use super::LinkGesture;

    #[test]
    fn only_real_down_and_same_occurrence_release_activate() {
        let mut gesture = LinkGesture::<u32, &str>::default();
        assert_eq!(gesture.release(Some(&1)), None);
        assert!(gesture.down(Some(1), "actual down"));
        assert_eq!(gesture.release(Some(&2)), None);
        assert_eq!(gesture.release(Some(&1)), None);
        gesture.down(Some(1), "actual down");
        assert_eq!(gesture.release(Some(&1)), Some("actual down"));
    }

    #[test]
    fn leaving_drag_blur_or_virtualization_cancel_without_rearming() {
        for (target, cancelled) in [(None, false), (Some(2), false), (Some(1), true)] {
            let mut gesture = LinkGesture::<u32, ()>::default();
            gesture.down(Some(1), ());
            gesture.reconcile(target, cancelled);
            gesture.reconcile(Some(1), false);
            assert_eq!(gesture.release(Some(&1)), None);
        }
    }

    #[test]
    fn stationary_pointer_reconciles_and_unchanged_hover_is_a_noop() {
        let mut gesture = LinkGesture::<u32, ()>::default();
        assert!(gesture.reconcile(Some(1), false));
        assert!(!gesture.reconcile(Some(1), false));
        assert!(gesture.reconcile(Some(2), false));
        assert!(gesture.reconcile(None, false));
        assert!(!gesture.reconcile(None, false));
    }
}
