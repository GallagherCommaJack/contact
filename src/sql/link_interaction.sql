INSERT INTO
  interactions(
    id,
    geo,
    symptom_id,
    ts_occurred,
    ts_uploaded
  )
VALUES($id, $geo, $symptom_id, $ts_occurred, now())

