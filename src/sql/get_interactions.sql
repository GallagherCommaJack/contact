SELECT
  id, geo, symptom_id
WHERE
  ts_uploaded BETWEEN $last_check AND now()
  AND geo LIKE $geo
