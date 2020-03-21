INSERT INTO confirmed_exposures (
  donor_id,
  recipient_id,
  total_minutes,
  close_contact
)

VALUES(
    $1,
    $2,
    $3,
    $4
);


INSERT INTO transmission_chains (
  left_id,
  right_id,
  relationship,
  certainty
)
VALUES (
    $1,
    $2,
    1, -- possible link, not confirmed
    $5
);

UPDATE cases 
SET exposed_cnt = exposed_cnt + 1
WHERE id = $1;
