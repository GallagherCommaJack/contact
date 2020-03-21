INSERT INTO cases(
    id,
    ts_exposure,
    ts_symptomatic,
    ts_resolved 
)
VALUES(
    $1,
    $2,
    $3,
    $4
)
