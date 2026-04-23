-- Add migration script here
ALTER TABLE skill_condition_types ALTER COLUMN description DROP NOT NULL;
ALTER TABLE skill_condition_types ALTER COLUMN example DROP NOT NULL;