use bytes::Bytes;
use deadpool_redis::Config;
use futures::TryFutureExt;
use once_cell::sync::Lazy;
use serde::*;
use std::{collections::HashSet, ops::DerefMut};
use warp::Filter;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

static POOL: Lazy<deadpool_redis::Pool> = Lazy::new(|| {
    let cfg = Config::from_env("REDIS").expect("need redis config");
    let pool = cfg.create_pool().expect("failed to create connection pool");
    pool
});

async fn link_interactions(uuids: &[&str], symptom_id: &str) -> Result<(), Error> {
    const SECONDS_IN_WEEK: usize = 7 * 24 * 60 * 60;

    let mut conn = POOL.get().await?;

    let mut pipe = redis::pipe();
    for uuid in uuids {
        pipe.set(*uuid, symptom_id);
        pipe.expire(*uuid, SECONDS_IN_WEEK);
    }
    pipe.query_async(conn.deref_mut().deref_mut()).await?;

    Ok(())
}

async fn append_symptoms(symptom_id: &str, symptoms: &[&str]) -> Result<(), Error> {
    let mut conn = POOL.get().await?;

    let mut pipe = redis::pipe();
    for symptom in symptoms {
        pipe.rpush(symptom_id, *symptom);
    }
    pipe.query_async(conn.deref_mut().deref_mut()).await?;

    Ok(())
}

async fn get_exposures(uuids: &[&str]) -> Result<Vec<Vec<String>>, Error> {
    let mut conn = POOL.get().await?;

    let sids: HashSet<Option<String>> = {
        let mut pipe = redis::pipe();
        for uuid in uuids {
            pipe.get(*uuid);
        }
        pipe.query_async(conn.deref_mut().deref_mut()).await?
    };

    let out: Vec<Vec<String>> = {
        let mut pipe = redis::pipe();

        for sid in sids.into_iter().filter_map(|x| x) {
            pipe.lrange(sid, 0, -1);
        }

        pipe.query_async(conn.deref_mut().deref_mut()).await?
    };

    Ok(out)
}

#[derive(Serialize, Deserialize)]
struct LinkInteractions<'a> {
    #[serde(borrow)]
    uuids: Vec<&'a str>,
    #[serde(borrow)]
    symptom_id: &'a str,
}

#[derive(Serialize, Deserialize)]
struct AppendSymptoms<'a> {
    #[serde(borrow)]
    symptom_id: &'a str,
    #[serde(borrow)]
    symptoms: Vec<&'a str>,
}

#[derive(Serialize, Deserialize)]
struct GetExposures<'a> {
    #[serde(borrow)]
    uuids: Vec<&'a str>,
}

#[derive(Debug)]
struct BadJson;

impl warp::reject::Reject for BadJson {}

#[derive(Debug)]
struct QueryFailed;

impl warp::reject::Reject for QueryFailed {}

#[tokio::main]
async fn main() {
    let link_interactions = warp::path("link_interactions")
        .and(warp::body::bytes())
        .and_then(|body: Bytes| async move {
            let LinkInteractions { uuids, symptom_id } =
                serde_json::from_slice(&body).map_err(|_| warp::reject::custom(BadJson))?;
            link_interactions(&uuids, symptom_id)
                .map_err(|_| warp::reject::custom(QueryFailed))
                .await?;
            Ok::<_, warp::reject::Rejection>("ok")
        });

    let append_symptoms = warp::path("append_symptoms")
        .and(warp::body::bytes())
        .and_then(|body: Bytes| async move {
            let AppendSymptoms {
                symptom_id,
                symptoms,
            } = serde_json::from_slice(&body).map_err(|_| warp::reject::custom(BadJson))?;
            append_symptoms(symptom_id, &symptoms)
                .map_err(|_| warp::reject::custom(QueryFailed))
                .await?;
            Ok::<_, warp::reject::Rejection>("ok")
        });

    let from_interactions = warp::path("get_exposures")
        .and(warp::body::bytes())
        .and_then(|body: Bytes| async move {
            let GetExposures { uuids } =
                serde_json::from_slice(&body).map_err(|_| warp::reject::custom(BadJson))?;

            let exposures = get_exposures(&uuids)
                .map_err(|_| warp::reject::custom(QueryFailed))
                .await?;

            let reply =
                serde_json::to_string(&exposures).map_err(|_| warp::reject::custom(QueryFailed))?;

            Ok::<_, warp::reject::Rejection>(reply)
        });

    let routes = link_interactions
        .or(append_symptoms.boxed())
        .or(from_interactions.boxed());

    warp::serve(routes).bind(([0u8, 0, 0, 0], 8080)).await;
}
