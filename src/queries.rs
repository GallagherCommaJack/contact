use super::*;
use chrono::*;
use deadpool_postgres::ClientWrapper as Client;
use futures::{
    future,
    stream::{self, Stream, StreamExt, TryStreamExt},
};
use tokio_postgres::{types::Type, Row, RowStream};

const CONCURRENT_REQS: usize = 10;

pub async fn get_interactions<Tz: TimeZone>(
    conn: &Client,
    last_check: DateTime<Tz>,
    geos: &[&str],
    interaction_ids: &[&str],
) -> Result<Vec<String>, Error> {
    let stmt = conn
        .prepare_typed(
            sql!("get_interactions"),
            types!(TIMESTAMP, TEXT_ARRAY, TEXT_ARRAY),
        )
        .await?;

    let params: &[&dyn postgres_types::ToSql] = params!(last_check, geos, interaction_ids);
    let rows = conn.query_raw(&stmt, params.iter().map(|x| *x)).await?;
    let res = rows
        .and_then(|row| future::ready(row.try_get::<_, String>("id")))
        .try_collect()
        .await?;

    Ok(res)
}

pub async fn confirm_interactions(conn: &Client, uuids: &[&str]) -> Result<Vec<String>, Error> {
    let stmt = conn
        .prepare_typed(sql!("confirm_interactions"), types!(TEXT_ARRAY))
        .await?;

    let params: &[&dyn postgres_types::ToSql] = params!(uuids);
    let rows = conn.query_raw(&stmt, params.iter().map(|x| *x)).await?;
    let res = rows
        .and_then(|row| future::ready(row.try_get::<_, String>("case_id")))
        .try_collect()
        .await?;

    Ok(res)
}

#[derive(Serialize, Deserialize)]
pub struct Symptom {
    pub symptom: String,
    pub ts: DateTime<Utc>,
}

pub async fn get_symptoms(conn: &Client, id: &str) -> Result<Vec<Symptom>, Error> {
    let stmt = conn
        .prepare_typed(sql!("get_symptoms"), types!(TEXT))
        .await?;

    let params: &[&dyn postgres_types::ToSql] = params!(id);
    let rows = conn.query_raw(&stmt, params.iter().map(|x| *x)).await?;
    let res = rows
        .and_then(|row| async move {
            Ok(Symptom {
                symptom: row.try_get("symptom")?,
                ts: row.try_get("ts")?,
            })
        })
        .try_collect()
        .await?;

    Ok(res)
}

pub async fn add_case<Tz1: TimeZone, Tz2: TimeZone, Tz3: TimeZone>(
    conn: &Client,
    id: &str,
    ts_exposure: Option<DateTime<Tz1>>,
    ts_symptomatic: DateTime<Tz2>,
    ts_resolved: Option<DateTime<Tz3>>,
) -> Result<u64, Error> {
    let stmt = conn
        .prepare_typed(
            sql!("add_case"),
            types!(TEXT, TIMESTAMPTZ, TIMESTAMPTZ, TIMESTAMPTZ),
        )
        .await?;

    let res = conn
        .execute(&stmt, params!(id, ts_exposure, ts_symptomatic, ts_resolved))
        .await?;

    Ok(res)
}
