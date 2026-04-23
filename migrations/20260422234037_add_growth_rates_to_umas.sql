-- Add migration script here
ALTER TABLE umas
    ADD COLUMN growth_speed   INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN growth_stamina INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN growth_power   INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN growth_guts    INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN growth_wit     INTEGER NOT NULL DEFAULT 0;