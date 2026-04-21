-- Add migration script here
-- Enums
CREATE TYPE skill_category AS ENUM (
    'Green', 'Recovery', 'Velocity', 'Acceleration', 'Movement',
    'Gate', 'Vision', 'SpeedDebuff', 'AccelDebuff', 'FrenzyDebuff',
    'StaminaDrain', 'VisionDebuff', 'Purple', 'Scenario'
);

CREATE TYPE skill_rarity AS ENUM (
    'Normal', 'Rare', 'Unique', 'Evolution'
);

CREATE TYPE uma_rarity AS ENUM (
    'R', 'SR', 'SSR'
);

CREATE TYPE aptitude_level AS ENUM (
    'A', 'B', 'C', 'D', 'E', 'F', 'G'
);

CREATE TYPE skill_acquisition AS ENUM (
    'Unique', 'Innate', 'Awakening', 'Event', 'Evolution'
);

-- Tables
CREATE TABLE skills (
    id            INTEGER PRIMARY KEY,
    name          TEXT NOT NULL,
    description   TEXT NOT NULL,
    category      skill_category NOT NULL,
    rarity        skill_rarity NOT NULL,
    sp_cost       INTEGER NOT NULL,
    eval_points   INTEGER NOT NULL
);

CREATE TABLE umas (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    subtitle        TEXT NOT NULL,
    rarity          uma_rarity NOT NULL,

    -- base_stats
    stat_speed      INTEGER NOT NULL,
    stat_stamina    INTEGER NOT NULL,
    stat_power      INTEGER NOT NULL,
    stat_guts       INTEGER NOT NULL,
    stat_wit        INTEGER NOT NULL,

    -- surface aptitudes
    apt_turf        aptitude_level NOT NULL,
    apt_dirt        aptitude_level NOT NULL,

    -- distance aptitudes
    apt_short       aptitude_level NOT NULL,
    apt_mile        aptitude_level NOT NULL,
    apt_medium      aptitude_level NOT NULL,
    apt_long        aptitude_level NOT NULL,

    -- strategy aptitudes
    apt_front       aptitude_level NOT NULL,
    apt_pace        aptitude_level NOT NULL,
    apt_late        aptitude_level NOT NULL,
    apt_end         aptitude_level NOT NULL
);

CREATE TABLE uma_skills (
    uma_id        INTEGER NOT NULL REFERENCES umas(id),
    skill_id      INTEGER NOT NULL REFERENCES skills(id),
    acquisition   skill_acquisition NOT NULL,
    PRIMARY KEY (uma_id, skill_id)
);
