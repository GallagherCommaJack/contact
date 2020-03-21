CREATE TABLE clients (
  id                TEXT NOT NULL PRIMARY KEY,
  jwt_token         TEXT NOT NULL,
  ts_begin          TIMESTAMP with time zone NOT NULL,
  ts_last_seen      TIMESTAMP with time zone NOT NULL,
  platform_type     TEXT NOT NULL
);

CREATE INDEX clients_by_id ON clients USING HASH (id);

CREATE TABLE cases (
  id                TEXT NOT NULL PRIMARY KEY,
  ts_exposure       TIMESTAMP with time zone,
  ts_symptomatic    TIMESTAMP with time zone NOT NULL,
  ts_resolved       TIMESTAMP with time zone,
  donor_cnt         int NOT NULL default 0,
  exposed_cnt       int NOT NULL default 0
);

CREATE INDEX case_by_id ON cases USING HASH (id);

CREATE TABLE symptoms (
  case_id TEXT NOT NULL,
  symptom TEXT NOT NULL,
  ts      TIMESTAMP with time zone NOT NULL,
  PRIMARY KEY(case_id, symptom)
);

CREATE INDEX symptom_by_case_id ON symptoms USING HASH (case_id);
CREATE INDEX symptom_by_description ON symptoms(symptom);

CREATE TABLE interactions (
  id          TEXT NOT NULL PRIMARY KEY,
  qtr_id      TEXT NOT NULL,
  geo         TEXT NOT NULL,
  case_id  TEXT NOT NULL,
  ts_uploaded TIMESTAMP with time zone NOT NULL 
);

CREATE INDEX interactions_by_uploaded_geo_id ON interactions(ts_uploaded, geo, qtr_id);
CREATE INDEX interactions_by_case ON interactions USING HASH(case_id);

CREATE TABLE confirmed_exposures (
  id            SERIAL PRIMARY KEY,
  donor_id      TEXT REFERENCES clients(id),
  recipient_id  TEXT REFERENCES clients(id),
  total_minutes INTEGER,
  close_contact BOOLEAN
);

CREATE TABLE transmission_chains (
  left_id       TEXT NOT NULL,
  right_id      TEXT NOT NULL,
  relationship  INT NOT NULL,
  certainty     DOUBLE PRECISION NOT NULL,
  PRIMARY KEY(left_id, right_id)
);

CREATE INDEX transmission_chain_by_left_id ON transmission_chains USING HASH (left_id);
CREATE INDEX transmission_chain_by_right_id ON transmission_chains USING HASH (right_id);
