SELECT
  symptom, ts
FROM
  symptoms
WHERE
  id = $1  
