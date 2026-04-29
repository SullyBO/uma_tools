-- Add migration script here
ALTER TABLE public.uma_skills DROP CONSTRAINT uma_skills_pkey;
ALTER TABLE public.uma_skills ADD CONSTRAINT uma_skills_pkey PRIMARY KEY (uma_id, skill_id, acquisition);