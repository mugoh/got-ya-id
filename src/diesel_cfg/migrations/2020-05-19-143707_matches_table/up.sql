-- Matched Identifications to Identification Claims
CREATE TABLE matched_identifications (
    id BIGSERIAL PRIMARY KEY,
    claim_id INTEGER REFERENCES claimed_identifications (id) ON DELETE CASCADE NOT NULL,
    identification_id INTEGER REFERENCES identifications (id) ON DELETE CASCADE NOT NULL,
    created_at timestamp without time zone not null default (now() at time zone 'utc')
);

CREATE UNIQUE INDEX matched_claim_id_unique ON matched_identifications (claim_id, identification_id);
