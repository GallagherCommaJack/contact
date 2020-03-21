INSERT INTO
  interactions(
    id,
    qtr_id,
    geo,
    symptom_id,
    ts_uploaded
  )
VALUES($id, left($id, 4), $geo, $symptom_id, now())

