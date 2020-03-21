SELECT
  id, geo, symptom_id
WHERE
  ts_uploaded BETWEEN $1 AND now()
  AND geo LIKE $2
  AND qtr_id IN ($3)