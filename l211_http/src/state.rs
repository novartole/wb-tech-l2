use crate::{error::AppError, model::Event};
use chrono::NaiveDate;
use std::future::Future;

#[derive(Default, Clone)]
pub struct AppState<T>
where
    T: StoreEvent,
{
    pub db: T,
}

impl<T> AppState<T>
where
    T: StoreEvent,
{
    pub fn new(db: T) -> Self {
        Self { db }
    }
}

pub trait StoreEvent: Clone {
    fn create_event(&self, event: Event) -> impl Future<Output = Result<Event, AppError>> + Send;

    fn update_event(&self, event: Event) -> impl Future<Output = Result<Event, AppError>> + Send;

    fn delete_event(&self, user_id: usize) -> impl Future<Output = Result<(), AppError>> + Send;

    fn events_for_day(
        &self,
        date: NaiveDate,
        user_id: Option<usize>,
    ) -> impl Future<Output = Result<Option<Vec<Event>>, AppError>> + Send;

    fn events_for_week(
        &self,
        date: NaiveDate,
        user_id: Option<usize>,
    ) -> impl Future<Output = Result<Option<Vec<Event>>, AppError>> + Send;

    fn events_for_month(
        &self,
        date: NaiveDate,
        user_id: Option<usize>,
    ) -> impl Future<Output = Result<Option<Vec<Event>>, AppError>> + Send;
}
