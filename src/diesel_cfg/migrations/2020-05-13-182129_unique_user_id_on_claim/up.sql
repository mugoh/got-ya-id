ALTER TABLE claimed_identifications
ADD CONSTRAINT unique_user_id_claims UNIQUE (user_id);
