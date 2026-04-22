-- Add migration script here
CREATE TYPE skill_operator AS ENUM (
    'Eq', 'NotEq', 'Gt', 'GtEq', 'Lt', 'LtEq'
);

CREATE TABLE skill_condition_types (
    id          SERIAL PRIMARY KEY,
    cond_key    TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    example     TEXT
);

CREATE TABLE skill_effects (
    id          SERIAL PRIMARY KEY,
    skill_id    INTEGER NOT NULL REFERENCES skills(id)
);

CREATE TABLE skill_effect_stats (
    id          SERIAL PRIMARY KEY,
    effect_id   INTEGER NOT NULL REFERENCES skill_effects(id),
    stat_key    TEXT NOT NULL,
    stat_val    TEXT NOT NULL
);

CREATE TABLE skill_conditions (
    id                  SERIAL PRIMARY KEY,
    effect_id           INTEGER NOT NULL REFERENCES skill_effects(id),
    condition_type_id   INTEGER NOT NULL REFERENCES skill_condition_types(id),
    is_precondition     BOOLEAN NOT NULL DEFAULT false,
    operator            skill_operator NOT NULL,
    cond_val            TEXT NOT NULL
);