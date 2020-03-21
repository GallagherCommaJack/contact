INSERT INTO
  interactions(
    id,
    geo,
    symptom_id,
    ts_uploaded
  )
VALUES($id, $geo, $symptom_id, now())

