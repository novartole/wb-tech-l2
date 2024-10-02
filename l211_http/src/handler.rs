use crate::{
    error::AppError,
    json::AppJson,
    model::{Event, EventDate},
    state::{AppState, StoreEvent},
};
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use tracing::trace;

pub async fn create_event<S: StoreEvent>(
    State(state): State<AppState<S>>,
    AppJson(payload): AppJson<Event>,
) -> Result<AppJson<Event>, AppError> {
    trace!(event = ?payload, "creating event");
    state.db.create_event(payload).await.map(AppJson)
}

pub async fn update_event<S: StoreEvent>(
    State(state): State<AppState<S>>,
    AppJson(payload): AppJson<Event>,
) -> Result<AppJson<Event>, AppError> {
    trace!(event = ?payload, "updating event");
    state.db.update_event(payload).await.map(AppJson)
}

pub async fn delete_event<S: StoreEvent>(
    State(state): State<AppState<S>>,
    AppJson(Event { user_id, .. }): AppJson<Event>,
) -> Result<(), AppError> {
    trace!(user_id, "deleting event");
    state.db.delete_event(user_id).await
}

#[derive(Deserialize)]
pub struct UserIdParam {
    #[serde(default)]
    pub user_id: Option<usize>,
}

pub async fn get_events_for_day<S: StoreEvent>(
    State(state): State<AppState<S>>,
    Path(EventDate { inner: date }): Path<EventDate>,
    Query(UserIdParam { user_id }): Query<UserIdParam>,
) -> Result<AppJson<Vec<Event>>, AppError> {
    trace!(day = %date, "getting events for day");
    state
        .db
        .events_for_day(date, user_id)
        .await?
        .map(AppJson)
        .ok_or(AppError::ServiceUnavailable("no events for this day"))
}

pub async fn get_events_for_week<S: StoreEvent>(
    State(state): State<AppState<S>>,
    Path(EventDate { inner: date }): Path<EventDate>,
    Query(UserIdParam { user_id }): Query<UserIdParam>,
) -> Result<AppJson<Vec<Event>>, AppError> {
    trace!(day = %date, "getting events for week of day");
    state
        .db
        .events_for_week(date, user_id)
        .await?
        .map(AppJson)
        .ok_or(AppError::ServiceUnavailable("no events for this week"))
}

pub async fn get_events_for_month<S: StoreEvent>(
    State(state): State<AppState<S>>,
    Path(EventDate { inner: date }): Path<EventDate>,
    Query(UserIdParam { user_id }): Query<UserIdParam>,
) -> Result<AppJson<Vec<Event>>, AppError> {
    trace!(day = %date, "getting events for month of day");
    state
        .db
        .events_for_month(date, user_id)
        .await?
        .map(AppJson)
        .ok_or(AppError::ServiceUnavailable("no events for this month"))
}
