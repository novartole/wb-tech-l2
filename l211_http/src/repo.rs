use crate::{error::AppError, model::Event, state::StoreEvent};
use chrono::{Datelike, Months, NaiveDate, Weekday};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum InMemoryStorageError {}

#[derive(Clone, Default)]
pub struct InMemoryStorage {
    pub inner: Arc<Mutex<HashMap<usize, Event>>>,
}

impl StoreEvent for InMemoryStorage {
    async fn create_event(&self, event @ Event { user_id, .. }: Event) -> Result<Event, AppError> {
        if self.inner.lock().await.insert(user_id, event).is_some() {
            return Err(AppError::ServiceUnavailable("already exists"));
        }

        self.inner
            .lock()
            .await
            .get(&user_id)
            .cloned()
            .ok_or(AppError::ServiceUnavailable("cannot get event"))
    }

    async fn update_event(&self, Event { user_id, date }: Event) -> Result<Event, AppError> {
        self.inner
            .lock()
            .await
            .entry(user_id)
            .and_modify(|event| event.date = date);

        self.inner
            .lock()
            .await
            .get(&user_id)
            .cloned()
            .ok_or(AppError::ServiceUnavailable("cannot get event"))
    }

    async fn delete_event(&self, user_id: usize) -> Result<(), AppError> {
        self.inner
            .lock()
            .await
            .remove(&user_id)
            .ok_or(AppError::ServiceUnavailable("nothing to delete"))
            .map(|_| ())
    }

    async fn events_for_day(
        &self,
        date: NaiveDate,
        user_id: Option<usize>,
    ) -> Result<Option<Vec<Event>>, AppError> {
        let events: Vec<_> = self
            .inner
            .lock()
            .await
            .values()
            .filter(|event| {
                let mut pass = event.date.inner == date;

                if let Some(user_id) = user_id {
                    pass &= event.user_id == user_id;
                }

                pass
            })
            .cloned()
            .collect();

        if events.is_empty() {
            return Ok(None);
        }

        Ok(Some(events))
    }

    async fn events_for_week(
        &self,
        date: NaiveDate,
        user_id: Option<usize>,
    ) -> Result<Option<Vec<Event>>, AppError> {
        let events: Vec<_> = self
            .inner
            .lock()
            .await
            .values()
            .filter(|event| {
                let mut pass = date.week(Weekday::Mon).days().contains(&event.date.inner);

                if let Some(user_id) = user_id {
                    pass &= event.user_id == user_id;
                }

                pass
            })
            .cloned()
            .collect();

        if events.is_empty() {
            return Ok(None);
        }

        Ok(Some(events))
    }

    async fn events_for_month(
        &self,
        date: NaiveDate,
        user_id: Option<usize>,
    ) -> Result<Option<Vec<Event>>, AppError> {
        let events: Vec<_> = self
            .inner
            .lock()
            .await
            .values()
            .filter(|event| {
                let start = date.with_day0(0).unwrap();
                let end = start
                    .checked_add_months(Months::new(1))
                    .unwrap()
                    .with_day0(0)
                    .unwrap();
                let mut pass = start
                    .iter_days()
                    .take_while(|day| *day != end)
                    .any(|day| day == event.date.inner);

                if let Some(user_id) = user_id {
                    pass &= event.user_id == user_id;
                }

                pass
            })
            .cloned()
            .collect();

        if events.is_empty() {
            return Ok(None);
        }

        Ok(Some(events))
    }
}
