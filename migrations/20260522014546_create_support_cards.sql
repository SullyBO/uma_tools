-- Add migration script here
CREATE TYPE support_card_type AS ENUM ('speed', 'stamina', 'power', 'guts', 'wit', 'friend', 'group');
CREATE TYPE support_card_rarity AS ENUM ('r', 'sr', 'ssr');
CREATE TYPE support_skill_acquisition AS ENUM ('event', 'hint');

CREATE TABLE support_cards (
    support_id        INTEGER PRIMARY KEY,
    char_name         TEXT NOT NULL,
    title             TEXT NOT NULL,
    card_type         support_card_type NOT NULL,
    rarity            support_card_rarity NOT NULL,
    is_welfare        BOOLEAN NOT NULL DEFAULT FALSE,
    release_en        DATE,
    is_predicted_date BOOLEAN NOT NULL DEFAULT FALSE,
    unique_effect     TEXT
);

CREATE TABLE support_card_effects (
    support_id  INTEGER NOT NULL REFERENCES support_cards(support_id),
    effect_id   INTEGER NOT NULL,
    lb0         INTEGER,
    lb1         INTEGER,
    lb2         INTEGER,
    lb3         INTEGER,
    mlb         INTEGER,
    PRIMARY KEY (support_id, effect_id)
);

CREATE TABLE support_card_skills (
    support_id  INTEGER NOT NULL REFERENCES support_cards(support_id),
    skill_id    INTEGER NOT NULL REFERENCES skills(id),
    acquisition support_skill_acquisition NOT NULL,
    PRIMARY KEY (support_id, skill_id, acquisition)
);

ALTER TABLE public.skill_triggers
    ADD COLUMN scaling text;