-- Add migration script here
-- Enums
CREATE TYPE skill_category AS ENUM (
    'green',
    'recovery',
    'unique_recovery',
    'velocity',
    'acceleration',
    'movement',
    'gate',
    'vision',
    'speed_debuff',
    'accel_debuff',
    'frenzy_debuff',
    'stamina_drain',
    'vision_debuff',
    'purple',
    'scenario',
    'unique',
    'zenkai'
);

CREATE TYPE skill_rarity AS ENUM (
    'normal',
    'rare',
    'unique',
    'evolution'
);

CREATE TYPE skill_acquisition AS ENUM (
    'unique',
    'innate',
    'awakening',
    'event',
    'evolution'
);

CREATE TYPE skill_operator AS ENUM (
    'eq',
    'not_eq',
    'gt',
    'gt_eq',
    'lt',
    'lt_eq'
);

CREATE TYPE uma_rarity AS ENUM (
    'r',
    'sr',
    'ssr'
);

CREATE TYPE aptitude_level AS ENUM (
    'a', 'b', 'c', 'd', 'e', 'f', 'g'
);

-- Skills
CREATE TABLE public.skills (
    id integer NOT NULL,
    name text NOT NULL,
    ingame_description text NOT NULL DEFAULT '',
    category skill_category NOT NULL,
    rarity skill_rarity NOT NULL,
    sp_cost integer NOT NULL DEFAULT 0,
    is_jp_only boolean NOT NULL DEFAULT false,
    CONSTRAINT skills_pkey PRIMARY KEY (id)
);

CREATE TABLE public.skill_triggers (
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY,
    skill_id integer NOT NULL,
    CONSTRAINT skill_triggers_pkey PRIMARY KEY (id),
    CONSTRAINT skill_triggers_skill_id_fkey FOREIGN KEY (skill_id)
        REFERENCES public.skills(id) ON DELETE CASCADE
);

CREATE TABLE public.skill_trigger_effects (
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY,
    trigger_id integer NOT NULL,
    effect_type text NOT NULL,
    effect_value integer,
    CONSTRAINT skill_trigger_effects_pkey PRIMARY KEY (id),
    CONSTRAINT skill_trigger_effects_trigger_id_fkey FOREIGN KEY (trigger_id)
        REFERENCES public.skill_triggers(id) ON DELETE CASCADE
);

CREATE TABLE public.skill_trigger_conditions (
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY,
    trigger_id integer NOT NULL,
    cond_key text NOT NULL,
    operator skill_operator NOT NULL,
    cond_val text NOT NULL,
    is_precondition boolean NOT NULL DEFAULT false,
    is_or boolean NOT NULL DEFAULT false,
    CONSTRAINT skill_trigger_conditions_pkey PRIMARY KEY (id),
    CONSTRAINT skill_trigger_conditions_trigger_id_fkey FOREIGN KEY (trigger_id)
        REFERENCES public.skill_triggers(id) ON DELETE CASCADE
);

CREATE TABLE public.skill_condition_types (
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY,
    cond_key text NOT NULL UNIQUE,
    description text,
    example text,
    CONSTRAINT skill_condition_types_pkey PRIMARY KEY (id)
);

-- Uma
CREATE TABLE public.umas (
    id integer NOT NULL,
    name text NOT NULL,
    subtitle text NOT NULL,
    rarity uma_rarity NOT NULL,
    stat_speed integer NOT NULL,
    stat_stamina integer NOT NULL,
    stat_power integer NOT NULL,
    stat_guts integer NOT NULL,
    stat_wit integer NOT NULL,
    apt_turf aptitude_level NOT NULL,
    apt_dirt aptitude_level NOT NULL,
    apt_short aptitude_level NOT NULL,
    apt_mile aptitude_level NOT NULL,
    apt_medium aptitude_level NOT NULL,
    apt_long aptitude_level NOT NULL,
    apt_front aptitude_level NOT NULL,
    apt_pace aptitude_level NOT NULL,
    apt_late aptitude_level NOT NULL,
    apt_end aptitude_level NOT NULL,
    growth_speed integer NOT NULL DEFAULT 0,
    growth_stamina integer NOT NULL DEFAULT 0,
    growth_power integer NOT NULL DEFAULT 0,
    growth_guts integer NOT NULL DEFAULT 0,
    growth_wit integer NOT NULL DEFAULT 0,
    CONSTRAINT umas_pkey PRIMARY KEY (id)
);

CREATE TABLE public.uma_skills (
    uma_id integer NOT NULL,
    skill_id integer NOT NULL,
    acquisition skill_acquisition NOT NULL,
    evolved_from integer REFERENCES public.skills(id),
    CONSTRAINT uma_skills_pkey PRIMARY KEY (uma_id, skill_id),
    CONSTRAINT uma_skills_uma_id_fkey FOREIGN KEY (uma_id)
        REFERENCES public.umas(id) ON DELETE CASCADE,
    CONSTRAINT uma_skills_skill_id_fkey FOREIGN KEY (skill_id)
        REFERENCES public.skills(id)
);