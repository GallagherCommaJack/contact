INSERT INTO clients(
    id,
    jwt_token,
    ts_begin,
    ts_last_seen
    ,platform_type
)
VALUES(
    $1,
    $2,
    now(),
    now(),
    $3
)
