use super::*;
use chrono::*;

fn case_key(id: &str) -> String {
    format!("case:{}", id)
}

fn time_key(ts: chrono::DateTime<Utc>) -> String {
    format!("{}", ts.format("time:%Y:%j:%H"))
}

fn time_keys_since(ts: chrono::DateTime<Utc>) -> impl Iterator<Item = String> {
    let num_hours = Utc::now().signed_duration_since(ts).num_hours();
    (0..num_hours)
        .map(chrono::Duration::hours)
        .map(move |dur| time_key(ts + dur))
}

pub mod report_symptoms {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Req<'a> {
        symptoms: Vec<&'a str>,
        case_id: &'a str,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Resp {
        success: bool,
        ts: chrono::DateTime<Utc>,
    }

    impl<'a> Req<'a> {
        pub async fn handle(self) -> Result<Resp, Error> {
            let Req { symptoms, case_id } = self;

            let ts = chrono::Utc::now();
            let time_key = time_key(ts);
            let case_key = case_key(case_id);

            let mut pipe = redis::pipe();
            for symptom in symptoms {
                pipe.rpush(case_key.as_str(), symptom);
            }
            pipe.sadd(time_key, case_id);

            let mut conn = POOL.get().await?;
            let success = pipe
                .query_async::<_, ()>(conn.deref_mut().deref_mut())
                .await
                .is_ok();

            Ok(Resp { success, ts })
        }
    }
}

pub mod get_symptoms {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Req<'a> {
        case_id: &'a str,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Resp {
        symptoms: Vec<String>,
    }

    impl<'a> Req<'a> {
        pub async fn handle(self) -> Result<Resp, Error> {
            let Req { case_id } = self;
            let case_key = case_key(case_id);
            let cmd = redis::Cmd::lrange(case_key, 0, -1);
            let mut conn = POOL.get().await?;
            Ok(Resp {
                symptoms: cmd.query_async(conn.deref_mut().deref_mut()).await?,
            })
        }
    }
}

pub mod get_cases {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Req {
        since: chrono::DateTime<Utc>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Resp {
        case_ids: HashSet<String>,
    }

    impl Req {
        pub async fn handle(self) -> Result<Resp, Error> {
            let Req { since } = self;
            let keys = time_keys_since(since);

            let mut pipe = redis::pipe();
            for key in keys {
                pipe.smembers(key);
            }
            let mut conn = POOL.get().await?;
            let cases: Vec<Vec<String>> = pipe.query_async(conn.deref_mut().deref_mut()).await?;

            Ok(Resp {
                case_ids: cases.into_iter().flatten().collect(),
            })
        }
    }
}
