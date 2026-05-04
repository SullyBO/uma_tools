-- Add migration script here
ALTER TABLE umas
    ALTER COLUMN apt_turf TYPE text,
    ALTER COLUMN apt_dirt TYPE text,
    ALTER COLUMN apt_short TYPE text,
    ALTER COLUMN apt_mile TYPE text,
    ALTER COLUMN apt_medium TYPE text,
    ALTER COLUMN apt_long TYPE text,
    ALTER COLUMN apt_front TYPE text,
    ALTER COLUMN apt_pace TYPE text,
    ALTER COLUMN apt_late TYPE text,
    ALTER COLUMN apt_end TYPE text;

-- This is to make comparisons work in the correct order (e.g. a > b > g)
DROP TYPE aptitude_level;
CREATE TYPE aptitude_level AS ENUM ('g', 'f', 'e', 'd', 'c', 'b', 'a');

ALTER TABLE umas
    ALTER COLUMN apt_turf TYPE aptitude_level USING apt_turf::aptitude_level,
    ALTER COLUMN apt_dirt TYPE aptitude_level USING apt_dirt::aptitude_level,
    ALTER COLUMN apt_short TYPE aptitude_level USING apt_short::aptitude_level,
    ALTER COLUMN apt_mile TYPE aptitude_level USING apt_mile::aptitude_level,
    ALTER COLUMN apt_medium TYPE aptitude_level USING apt_medium::aptitude_level,
    ALTER COLUMN apt_long TYPE aptitude_level USING apt_long::aptitude_level,
    ALTER COLUMN apt_front TYPE aptitude_level USING apt_front::aptitude_level,
    ALTER COLUMN apt_pace TYPE aptitude_level USING apt_pace::aptitude_level,
    ALTER COLUMN apt_late TYPE aptitude_level USING apt_late::aptitude_level,
    ALTER COLUMN apt_end TYPE aptitude_level USING apt_end::aptitude_level;