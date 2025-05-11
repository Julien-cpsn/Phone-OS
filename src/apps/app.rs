use crate::events::AppEvent;
use crate::phone::AppAccessible;
use mousefood::prelude::{Frame, Rect};
use std::any::Any;
use crate::state::PhoneState;

pub trait App: Any {
    fn new_boxed() -> Box<dyn App> where Self: Sized;
    fn get_name(&self) -> &'static str;
    fn render(&mut self, app_accessible: &mut AppAccessible, frame: &mut Frame, area: Rect) -> anyhow::Result<Vec<(Rect, Box<dyn AppEvent>)>>;

    fn handle_event(&mut self, app_accessible: &mut AppAccessible, event: &Box<dyn AppEvent>) -> anyhow::Result<Option<PhoneState>>;
}

pub struct AppImpl<T: AppHandler> {
    inner: T,
}

// This is a new trait that handles specific event types
pub trait AppHandler: Any {
    type Event: AppEvent;

    fn new() -> Self where Self: Sized;
    fn get_name(&self) -> &'static str;
    fn render(&mut self, app_accessible: &mut AppAccessible, frame: &mut Frame, area: Rect) -> anyhow::Result<Vec<(Rect, Box<dyn AppEvent>)>>;
    fn handle_event(&mut self, app_accessible: &mut AppAccessible, event: &Self::Event) -> anyhow::Result<Option<PhoneState>>;
}

impl<T: AppHandler + 'static> App for AppImpl<T> {
    fn new_boxed() -> Box<dyn App> where Self: Sized {
        Box::new(AppImpl {
            inner: T::new(),
        })
    }

    fn get_name(&self) -> &'static str {
        self.inner.get_name()
    }

    fn render(&mut self, app_accessible: &mut AppAccessible, frame: &mut Frame, area: Rect) -> anyhow::Result<Vec<(Rect, Box<dyn AppEvent>)>> {
        self.inner.render(app_accessible, frame, area)
    }

    fn handle_event(&mut self, app_accessible: &mut AppAccessible, event: &Box<dyn AppEvent>) -> anyhow::Result<Option<PhoneState>> {
        if let Some(concrete_event) = event.as_any().downcast_ref::<T::Event>() {
            self.inner.handle_event(app_accessible, concrete_event)
        }
        else {
            Ok(None)
        }
    }
}