DROP TABLE IF EXISTS emails;
DROP TABLE IF EXISTS email; -- Initial name. Err - column with same name

ALTER TABLE users
ADD COLUMN email VARCHAR UNIQUE;

CREATE UNIQUE INDEX IF NOT EXISTS user_creds on USERS (username, email);
