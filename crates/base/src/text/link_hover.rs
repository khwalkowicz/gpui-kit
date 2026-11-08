use std::{cell::RefCell, rc::Rc, sync::Arc};

use gpui::{App, SharedString};

pub(super) type LinkHoverHandler = dyn Fn(Option<&SharedString>, &mut App) + Send + Sync;

#[derive(Default)]
struct HoverState {
    handler: Option<Arc<LinkHoverHandler>>,
    destination: Option<SharedString>,
    epoch: u64,
    delivered: (u64, Option<SharedString>),
    pending: bool,
}

#[derive(Clone, Default)]
pub(super) struct LinkHover(Rc<RefCell<HoverState>>);

impl LinkHover {
    pub fn set_handler(&self, handler: Option<Arc<LinkHoverHandler>>) {
        let mut state = self.0.borrow_mut();
        state.handler = handler;
        if state.handler.is_none() {
            state.destination = None;
            state.delivered = (state.epoch, None);
        }
    }

    pub fn is_observed(&self) -> bool {
        self.0.borrow().handler.is_some()
    }

    pub fn reset(&self, cx: &mut App) {
        self.0.borrow_mut().epoch += 1;
        self.update(None, cx);
    }

    pub fn update(&self, destination: Option<SharedString>, cx: &mut App) {
        let mut state = self.0.borrow_mut();
        if state.handler.is_none() || state.destination == destination {
            return;
        }
        state.destination = destination;
        if state.pending {
            return;
        }
        state.pending = true;
        let hover = self.clone();
        cx.defer(move |cx| {
            let (handler, destination) = {
                let mut state = hover.0.borrow_mut();
                state.pending = false;
                let current = (state.epoch, state.destination.clone());
                if state.delivered == current {
                    return;
                }
                state.delivered = current;
                (state.handler.clone(), state.destination.clone())
            };
            if let Some(handler) = handler {
                handler(destination.as_ref(), cx);
            }
        });
    }
}
