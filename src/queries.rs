use super::*;
use chrono::*;
use deadpool_postgres::ClientWrapper as Client;
use futures::stream::{self, Stream, StreamExt, TryStreamExt};
use tokio_postgres::{types::Type, Row, RowStream};

const CONCURRENT_REQS: usize = 10;

async fn raw_get_interactions<'a>(
    conn: &'a Client,
    last_check: DateTime<Utc>,
    geos: &[&str],
    interaction_ids: &[&str]
) -> Result<Vec<Row>, Error> {
    let stmt = conn
        .prepare_typed(sql!("get_interactions"), types!(TIMESTAMP, TEXT))
        .await?;

    let res = stream::iter(geos)
        .map(|geo| {
            let stmt = &stmt;
            async move {
                let params: &[&dyn postgres_types::ToSql] = params!(last_check, geo, interaction_ids);
                conn.query_raw(stmt, params.iter().map(|x| *x)).await
            }
        })
        .buffer_unordered(CONCURRENT_REQS)
        .try_flatten()
        .try_collect()
        .await?;

    Ok(res)
}

async fn raw_confirm_interactions<'a>(
    conn: &'a Client,
    uuids: &[&str],
) -> Result<Vec<Row>, Error> {
    let stmt = conn
        .prepare_typed(sql!("confirm_interactions"), types!(TEXT))
        .await?;

    Ok(conn.query(&stmt, params!(uuids)).await?)
}   

async fn raw_get_symptoms(conn: &Client, id: &str) -> Result<Option<Row>, Error> {
    let stmt = conn
        .prepare_typed(sql!("get_symptoms"), types!(TEXT))
        .await?;

    Ok(conn.query_opt(&stmt, params!(id)).await?)
}
