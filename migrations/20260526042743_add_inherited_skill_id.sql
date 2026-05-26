-- Add migration script here
ALTER TABLE skills ADD COLUMN inherited_skill_id INTEGER REFERENCES skills(id);