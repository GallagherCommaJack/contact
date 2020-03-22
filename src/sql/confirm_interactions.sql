SELECT DISTINCT
  case_id
FROM
  interactions
WHERE
  id IN ($1)
