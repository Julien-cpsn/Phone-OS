use crate::events::{AppEvent, EventType};
use crate::phone::PhoneData;
use mousefood::prelude::{Frame, Rect};
use std::any::Any;
use async_trait::async_trait;
use log::info;
use crate::state::PhoneState;

#[derive(Debug)]
pub struct ClickableArea(pub Rect, pub Box<dyn AppEvent>);

#[async_trait]
pub trait App: Any + Send + Sync {
    fn new_boxed() -> Box<dyn App> where Self: Sized;
    fn app_name(&self) -> &'static str;
    fn init(&mut self, phone_data: &mut PhoneData) -> anyhow::Result<()>;
    fn render(&mut self, phone_data: &mut PhoneData, frame: &mut Frame, area: Rect) -> anyhow::Result<EventType>;

    fn handle_event(&mut self, phone_data: &mut PhoneData, event: &Box<dyn AppEvent>) -> anyhow::Result<Option<PhoneState>>;
}

pub struct AppImpl<T: AppHandler> {
    inner: T,
}

unsafe impl<T: AppHandler + Send + 'static> Send for AppImpl<T> {}
unsafe impl<T: AppHandler + Send + 'static> Sync for AppImpl<T> {}

// This is a new trait that handles specific event types
#[async_trait]
pub trait AppHandler: Any {
    type Event: AppEvent;

    fn new() -> Self where Self: Sized;
    fn app_name(&self) -> &'static str;
    fn init(&mut self, phone_data: &mut PhoneData) -> anyhow::Result<()>;
    fn render(&mut self, phone_data: &mut PhoneData, frame: &mut Frame, area: Rect) -> anyhow::Result<EventType>;
    fn handle_event(&mut self, phone_data: &mut PhoneData, event: &Self::Event) -> anyhow::Result<Option<PhoneState>>;
}

#[async_trait]
impl<T: AppHandler + Send + 'static> App for AppImpl<T> {
    fn new_boxed() -> Box<dyn App> where Self: Sized {
        Box::new(AppImpl {
            inner: T::new(),
        })
    }

    fn app_name(&self) -> &'static str {
        self.inner.app_name()
    }

    fn init(&mut self, phone_data: &mut PhoneData) -> anyhow::Result<()> {
        self.inner.init(phone_data)
    }

    fn render(&mut self, app_accessible: &mut PhoneData, frame: &mut Frame, area: Rect) -> anyhow::Result<EventType> {
        self.inner.render(app_accessible, frame, area)
    }

    fn handle_event(&mut self, app_accessible: &mut PhoneData, event: &Box<dyn AppEvent>) -> anyhow::Result<Option<PhoneState>> {
        if let Some(concrete_event) = event.as_ref().as_any().downcast_ref::<T::Event>() {
            info!("{:?}", concrete_event);
            self.inner.handle_event(app_accessible, concrete_event)
        }
        else {
            Ok(None)
        }
    }
}