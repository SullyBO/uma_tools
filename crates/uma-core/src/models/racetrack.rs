use std::fmt;

#[derive(Default)]
pub struct Racetrack {
    name: String,
    pub courses: Vec<Course>,
    id: String,
}

pub struct Course {
    corners: Vec<Corner>,
    distance: i64,
    id: i64,
    inout: i64,
    laps: Vec<Lap>,
    length: i64,
    no_mans_land: Vec<NoMansLand>,
    overlaps: Vec<String>,
    phases: Vec<Phase>,
    position_keep_end: i64,
    slopes: Vec<Slope>,
    spurt_start: SpurtStart,
    stat_thresholds: Vec<i64>,
    straights: Vec<Straight>,
    terrain: i64,
    terrain_changes: Vec<TerrainChange>,
    turn: i64,
}

pub struct Corner {
    end: i64,
    number: i64,
    start: i64,
}

pub struct Lap {
    end: i64,
    lap: i64,
    start: i64,
}

pub struct NoMansLand {
    end: i64,
    start: i64,
}

pub struct Phase {
    end: i64,
    id: i64,
    start: i64,
}

pub struct Slope {
    end: i64,
    slope: i64,
    start: i64,
}

pub struct SpurtStart {
    lap: i64,
    location: Vec<Location>,
    meters: i64,
}

pub enum Location {
    Corner,
    Downhill,
    FinalCorner,
    FinalStraight,
    Straight,
    Uphill,
}

pub struct Straight {
    end: i64,
    front_type: i64,
    start: i64,
}

pub struct TerrainChange {
    start: i64,
    terrain: i64,
}

#[derive(Debug)]
pub enum RacetrackName {
    Sapporo,
    Hakodate,
    Niigata,
    Fukushima,
    Nakayama,
    Tokyo,
    Chukyo,
    Kyoto,
    Hanshin,
    Kokura,
    Ooi,
    Kawasaki,
    Funabashi,
    Morioka,
    Longchamp,
    SantaAnitaPark,
    DelMar,
}

impl fmt::Display for RacetrackName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = format!("{:?}", self);

        for (i, c) in name.chars().enumerate() {
            if i > 0 && c.is_uppercase() {
                f.write_str(" ")?;
            }
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}
