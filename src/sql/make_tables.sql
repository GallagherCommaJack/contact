CREATE TABLE symptoms (
  id      TEXT NOT NULL,
  symptom TEXT NOT NULL,
  ts      TIMESTAMP NOT NULL,
  PRIMARY KEY(symptom_id, symptom)
);

CREATE INDEX symptom_by_id ON symptoms USING HASH (id);
CREATE INDEX symptom_by_description ON symptoms(symptom);

CREATE TABLE interactions (
  id          TEXT NOT NULL PRIMARY KEY,
  geo         TEXT NOT NULL,
  symptom_id  TEXT NOT NULL,
  ts_uploaded TIMESTAMP NOT NULL 
);

CREATE INDEX interactions_by_uploaded_geo_occurred_id ON interactions(ts_uploaded, geo, ts_occurred, id);
CREATE INDEX interactions_by_case ON interactions USING HASH(symptom_id);
