ALTER TABLE matched_identifications
DROP CONSTRAINT IF EXISTS matched_claim_id_unique;

DROP TABLE matched_identifications;
