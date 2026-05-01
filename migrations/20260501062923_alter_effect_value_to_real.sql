-- Add migration script here
ALTER TABLE public.skill_trigger_effects
    ALTER COLUMN effect_value TYPE real;