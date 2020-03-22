use super::*;
use chrono::*;
use deadpool_postgres::ClientWrapper as Client;
use futures::{
    future,
    stream::{self, Stream, StreamExt, TryStreamExt},
};
use tokio_postgres::{types::Type, Row, RowStream};

const CONCURRENT_REQS: usize = 10;

pub async fn get_interactions(
    conn: &Client,
    last_check: DateTime<Utc>,
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

pub async fn add_case(
    conn: &Client,
    id: &str,
    ts_exposure: Option<DateTime<Utc>>,
    ts_symptomatic: DateTime<Utc>,
    ts_resolved: Option<DateTime<Utc>>,
) -> Result<(), Error> {
    let stmt = conn
        .prepare_typed(
            sql!("add_case"),
            types!(TEXT, TIMESTAMPTZ, TIMESTAMPTZ, TIMESTAMPTZ),
        )
        .await?;

    conn.execute(&stmt, params!(id, ts_exposure, ts_symptomatic, ts_resolved))
        .await?;

    Ok(())
}

pub async fn add_client(
    conn: &Client,
    id: &str,
    jwt: Option<&str>,
    platform: Option<&str>,
) -> Result<(), Error> {
    let stmt = conn
        .prepare_typed(sql!("add_client"), types!(TEXT, TEXT, TEXT))
        .await?;

    conn.execute(&stmt, params!(id, jwt, platform)).await?;

    Ok(())
}

pub async fn add_symptoms(
    conn: &Client,
    id: &str,
    symptoms: &[(&str, DateTime<Utc>)],
) -> Result<(), Error> {
    let stmt = conn
        .prepare_typed(sql!("add_symptom"), types!(TEXT, TEXT, TIMESTAMPTZ))
        .await?;

    let stmt = &stmt;
    stream::iter(symptoms)
        .map(Ok::<_, Error>)
        .try_for_each_concurrent(CONCURRENT_REQS, move |(symptom, ts)| async move {
            conn.execute(stmt, params!(id, symptom, ts)).await?;
            Ok(())
        })
        .await?;

    Ok(())
}

pub async fn clear_old_interactions(conn: &Client) -> Result<u64, Error> {
    let stmt = conn
        .prepare_typed(sql!("clear_old_interactions"), types!())
        .await?;

    let res = conn.execute(&stmt, params!()).await?;

    Ok(res)
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Exposure<'a> {
    donor: &'a str,
    recipient: &'a str,
    total_minutes: Option<i32>,
    close: Option<bool>,
    certainty: Option<f64>,
}

pub async fn confirm_exposures(conn: &Client, exposures: &[Exposure<'_>]) -> Result<(), Error> {
    let stmt = conn
        .prepare_typed(
            sql!("confirm_exposures"),
            types!(TEXT, TEXT, INT4, BOOL, FLOAT8),
        )
        .await?;

    let stmt = &stmt;
    stream::iter(exposures)
        .map(|x| Ok::<_, Error>(*x))
        .try_for_each_concurrent(
            CONCURRENT_REQS,
            move |Exposure {
                      donor,
                      recipient,
                      total_minutes,
                      close,
                      certainty,
                  }| async move {
                conn.execute(
                    stmt,
                    params!(donor, recipient, total_minutes, close, certainty),
                )
                .await?;
                Ok(())
            },
        )
        .await?;

    Ok(())
}
