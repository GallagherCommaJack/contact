use super::*;
use chrono::*;
use tokio_postgres::GenericClient;

#[derive(Serialize, Deserialize, Debug)]
pub struct Interaction<'a> {
    #[serde(borrow)]
    id: &'a str,
    #[serde(borrow)]
    geo: &'a str,
    #[serde(borrow)]
    symptom_id: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Symptom<'a> {
    #[serde(borrow)]
    id: &'a str,
    #[serde(borrow)]
    symptom: &'a str,
    ts: DateTime<Utc>,
}

async fn get_interactions<C: GenericClient>(
    conn: &C,
    last_check: DateTime<Utc>,
    geo: &str,
) -> Result<Vec<String>, Error> {
    todo!()
}
