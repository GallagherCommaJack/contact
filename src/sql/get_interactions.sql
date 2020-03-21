SELECT
  id
FROM
  interactions
WHERE
  ts_uploaded BETWEEN $1 AND now()
  AND geo LIKE $2
