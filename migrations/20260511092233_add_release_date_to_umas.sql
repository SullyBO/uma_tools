-- Add migration script here
ALTER TABLE umas ADD COLUMN release_date DATE;
ALTER TABLE umas ADD COLUMN is_predicted_date BOOLEAN NOT NULL DEFAULT FALSE;