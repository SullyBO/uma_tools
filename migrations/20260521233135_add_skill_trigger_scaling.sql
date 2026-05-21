-- Add migration script here
ALTER TABLE public.skill_triggers
    ADD COLUMN scaling text;