SELECT
  id
WHERE
  ts_uploaded BETWEEN $1 AND now()
  AND geo LIKE $2
