use chrono::{DateTime, Utc, Datelike, Timelike};
use astro::time;
use astro::planet;
use astro::lunar;
use astro::sun;
use astro::angle;

/// Represents the planets we care about for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Planet {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
}

impl Planet {
    #[allow(dead_code)]
    pub fn all() -> Vec<Planet> {
        vec![
            Planet::Sun,
            Planet::Moon,
            Planet::Mercury,
            Planet::Venus,
            Planet::Mars,
            Planet::Jupiter,
            Planet::Saturn,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Planet::Sun => "Sun",
            Planet::Moon => "Moon",
            Planet::Mercury => "Mercury",
            Planet::Venus => "Venus",
            Planet::Mars => "Mars",
            Planet::Jupiter => "Jupiter",
            Planet::Saturn => "Saturn",
        }
    }

    #[allow(dead_code)]
    pub fn domain(self) -> &'static str {
        match self {
            Planet::Sun => "Life Force & Critical Processes",
            Planet::Moon => "Emotions & Interactive Tasks",
            Planet::Mercury => "Communication & Network",
            Planet::Venus => "Harmony & Desktop/UI",
            Planet::Mars => "Energy & CPU-Intensive",
            Planet::Jupiter => "Expansion & Memory-Heavy",
            Planet::Saturn => "Structure & System Tasks",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Ophiuchus,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

/// Traditional 12-sign zodiac: equal 30° divisions
const TRADITIONAL_BOUNDARIES: &[(f64, ZodiacSign)] = &[
    (30.0, ZodiacSign::Aries),
    (60.0, ZodiacSign::Taurus),
    (90.0, ZodiacSign::Gemini),
    (120.0, ZodiacSign::Cancer),
    (150.0, ZodiacSign::Leo),
    (180.0, ZodiacSign::Virgo),
    (210.0, ZodiacSign::Libra),
    (240.0, ZodiacSign::Scorpio),
    (270.0, ZodiacSign::Sagittarius),
    (300.0, ZodiacSign::Capricorn),
    (330.0, ZodiacSign::Aquarius),
    (360.0, ZodiacSign::Pisces),
];

/// IAU 13-sign zodiac with Ophiuchus: unequal divisions per actual constellations
const IAU_13_SIGN_BOUNDARIES: &[(f64, ZodiacSign)] = &[
    (29.0, ZodiacSign::Pisces),
    (53.5, ZodiacSign::Aries),
    (90.5, ZodiacSign::Taurus),
    (118.0, ZodiacSign::Gemini),
    (138.0, ZodiacSign::Cancer),
    (174.0, ZodiacSign::Leo),
    (218.0, ZodiacSign::Virgo),
    (241.0, ZodiacSign::Libra),
    (248.0, ZodiacSign::Scorpio),
    (266.0, ZodiacSign::Ophiuchus),
    (299.5, ZodiacSign::Sagittarius),
    (327.5, ZodiacSign::Capricorn),
    (351.5, ZodiacSign::Aquarius),
    (360.0, ZodiacSign::Pisces),
];

impl ZodiacSign {
    pub fn from_longitude(longitude: f64, use_13_signs: bool) -> Self {
        let normalized = longitude.rem_euclid(360.0);
        let boundaries = if use_13_signs {
            IAU_13_SIGN_BOUNDARIES
        } else {
            TRADITIONAL_BOUNDARIES
        };

        for &(upper_bound, sign) in boundaries {
            if normalized < upper_bound {
                return sign;
            }
        }
        boundaries.last().map_or(ZodiacSign::Aries, |&(_, sign)| sign)
    }

    pub fn name(self) -> &'static str {
        match self {
            ZodiacSign::Aries => "Aries",
            ZodiacSign::Taurus => "Taurus",
            ZodiacSign::Gemini => "Gemini",
            ZodiacSign::Cancer => "Cancer",
            ZodiacSign::Leo => "Leo",
            ZodiacSign::Virgo => "Virgo",
            ZodiacSign::Libra => "Libra",
            ZodiacSign::Scorpio => "Scorpio",
            ZodiacSign::Ophiuchus => "Ophiuchus",
            ZodiacSign::Sagittarius => "Sagittarius",
            ZodiacSign::Capricorn => "Capricorn",
            ZodiacSign::Aquarius => "Aquarius",
            ZodiacSign::Pisces => "Pisces",
        }
    }

    pub fn element(self) -> Element {
        match self {
            ZodiacSign::Aries | ZodiacSign::Leo | ZodiacSign::Sagittarius | ZodiacSign::Ophiuchus => Element::Fire,
            ZodiacSign::Taurus | ZodiacSign::Virgo | ZodiacSign::Capricorn => Element::Earth,
            ZodiacSign::Gemini | ZodiacSign::Libra | ZodiacSign::Aquarius => Element::Air,
            ZodiacSign::Cancer | ZodiacSign::Scorpio | ZodiacSign::Pisces => Element::Water,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Fire,   // Energy, CPU
    Earth,  // Stability, Long-running
    Air,    // Communication, Network
    Water,  // Fluidity, Storage/DB
}

impl Element {
    pub fn name(self) -> &'static str {
        match self {
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Air => "Air",
            Element::Water => "Water",
        }
    }
}

/// Moon phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    NewMoon,        // 0-45°
    WaxingCrescent, // 45-90°
    FirstQuarter,   // 90-135°
    WaxingGibbous,  // 135-180°
    FullMoon,       // 180-225°
    WaningGibbous,  // 225-270°
    LastQuarter,    // 270-315°
    WaningCrescent, // 315-360°
}

impl MoonPhase {
    pub fn name(self) -> &'static str {
        match self {
            MoonPhase::NewMoon => "New Moon",
            MoonPhase::WaxingCrescent => "Waxing Crescent",
            MoonPhase::FirstQuarter => "First Quarter",
            MoonPhase::WaxingGibbous => "Waxing Gibbous",
            MoonPhase::FullMoon => "Full Moon",
            MoonPhase::WaningGibbous => "Waning Gibbous",
            MoonPhase::LastQuarter => "Last Quarter",
            MoonPhase::WaningCrescent => "Waning Crescent",
        }
    }

    /// Calculate moon phase from Sun-Moon angular separation
    pub fn from_angle(angle: f64) -> Self {
        let normalized = angle.rem_euclid(360.0);
        match normalized {
            a if a < 45.0 => MoonPhase::NewMoon,
            a if a < 90.0 => MoonPhase::WaxingCrescent,
            a if a < 135.0 => MoonPhase::FirstQuarter,
            a if a < 180.0 => MoonPhase::WaxingGibbous,
            a if a < 225.0 => MoonPhase::FullMoon,
            a if a < 270.0 => MoonPhase::WaningGibbous,
            a if a < 315.0 => MoonPhase::LastQuarter,
            _ => MoonPhase::WaningCrescent,
        }
    }
}

/// Planetary position information
#[derive(Debug, Clone)]
pub struct PlanetaryPosition {
    pub planet: Planet,
    pub longitude: f64,  // Ecliptic longitude in degrees
    pub sign: ZodiacSign,
    pub retrograde: bool,  // True if planet is in retrograde motion
    pub moon_phase: Option<MoonPhase>,  // Only for Moon - affects Interactive task scheduling
}

/// Convert chrono `DateTime` to astro crate's Date
fn to_astro_date(dt: &DateTime<Utc>) -> time::Date {
    let day_of_month = time::DayOfMonth {
        day: dt.day() as u8,
        hr: dt.hour() as u8,
        min: dt.minute() as u8,
        sec: f64::from(dt.second()),
        time_zone: 0.0,
    };

    // Safety: time::Month variants are explicitly valued 1-12, matching chrono's month()
    let month: time::Month = unsafe { std::mem::transmute(dt.month() as u8) };

    time::Date {
        year: dt.year() as i16,
        month,
        decimal_day: time::decimal_day(&day_of_month),
        cal_type: time::CalType::Gregorian,
    }
}

/// Detect if a planet is retrograde by comparing today's position with tomorrow's
/// Returns true if the planet is moving backward (westward) in the sky
fn is_retrograde(astro_planet: &planet::Planet, jd_today: f64) -> bool {
    let jd_tomorrow = jd_today + 1.0;

    let (pos_today, _) = planet::geocent_apprnt_ecl_coords(astro_planet, jd_today);
    let (pos_tomorrow, _) = planet::geocent_apprnt_ecl_coords(astro_planet, jd_tomorrow);

    let lon_today = angle::limit_to_360(pos_today.long.to_degrees());
    let lon_tomorrow = angle::limit_to_360(pos_tomorrow.long.to_degrees());

    // Handle 360° wraparound: if tomorrow crosses 0°, check if it's actually moving backward
    let delta = lon_tomorrow - lon_today;

    // If delta is large and positive (>180), planet crossed 360° going backward
    // If delta is negative and small (>-180), planet is moving backward normally
    if delta > 180.0 {
        true  // Crossed 360° while retrograde (e.g., 359° -> 1°)
    } else if delta < -180.0 {
        false  // Crossed 360° while direct (e.g., 1° -> 359°)
    } else {
        delta < 0.0  // Normal case: negative delta means retrograde
    }
}

/// Calculate planetary positions with retrograde detection (traditional 12-sign zodiac)
/// This is a convenience wrapper for `calculate_planetary_positions_with_zodiac(dt, false)`.
#[allow(dead_code)]  // Public API convenience wrapper
pub fn calculate_planetary_positions(dt: DateTime<Utc>) -> Vec<PlanetaryPosition> {
    calculate_planetary_positions_with_zodiac(dt, false)
}

/// Calculate planetary positions with configurable zodiac system
pub fn calculate_planetary_positions_with_zodiac(dt: DateTime<Utc>, use_13_signs: bool) -> Vec<PlanetaryPosition> {
    let date = to_astro_date(&dt);
    let jd = time::julian_day(&date);

    let mut positions = Vec::new();

    // Sun - geocentric ecliptic position (never retrograde)
    let (sun_ecl, _) = sun::geocent_ecl_pos(jd);
    let sun_lon_deg = angle::limit_to_360(sun_ecl.long.to_degrees());
    positions.push(PlanetaryPosition {
        planet: Planet::Sun,
        longitude: sun_lon_deg,
        sign: ZodiacSign::from_longitude(sun_lon_deg, use_13_signs),
        retrograde: false,
        moon_phase: None,
    });

    // Mercury
    let (merc_ecl, _) = planet::geocent_apprnt_ecl_coords(&planet::Planet::Mercury, jd);
    let merc_lon_deg = angle::limit_to_360(merc_ecl.long.to_degrees());
    positions.push(PlanetaryPosition {
        planet: Planet::Mercury,
        longitude: merc_lon_deg,
        sign: ZodiacSign::from_longitude(merc_lon_deg, use_13_signs),
        retrograde: is_retrograde(&planet::Planet::Mercury, jd),
        moon_phase: None,
    });

    // Venus
    let (venus_ecl, _) = planet::geocent_apprnt_ecl_coords(&planet::Planet::Venus, jd);
    let venus_lon_deg = angle::limit_to_360(venus_ecl.long.to_degrees());
    positions.push(PlanetaryPosition {
        planet: Planet::Venus,
        longitude: venus_lon_deg,
        sign: ZodiacSign::from_longitude(venus_lon_deg, use_13_signs),
        retrograde: is_retrograde(&planet::Planet::Venus, jd),
        moon_phase: None,
    });

    // Mars
    let (mars_ecl, _) = planet::geocent_apprnt_ecl_coords(&planet::Planet::Mars, jd);
    let mars_lon_deg = angle::limit_to_360(mars_ecl.long.to_degrees());
    positions.push(PlanetaryPosition {
        planet: Planet::Mars,
        longitude: mars_lon_deg,
        sign: ZodiacSign::from_longitude(mars_lon_deg, use_13_signs),
        retrograde: is_retrograde(&planet::Planet::Mars, jd),
        moon_phase: None,
    });

    // Jupiter
    let (jup_ecl, _) = planet::geocent_apprnt_ecl_coords(&planet::Planet::Jupiter, jd);
    let jup_lon_deg = angle::limit_to_360(jup_ecl.long.to_degrees());
    positions.push(PlanetaryPosition {
        planet: Planet::Jupiter,
        longitude: jup_lon_deg,
        sign: ZodiacSign::from_longitude(jup_lon_deg, use_13_signs),
        retrograde: is_retrograde(&planet::Planet::Jupiter, jd),
        moon_phase: None,
    });

    // Saturn
    let (sat_ecl, _) = planet::geocent_apprnt_ecl_coords(&planet::Planet::Saturn, jd);
    let sat_lon_deg = angle::limit_to_360(sat_ecl.long.to_degrees());
    positions.push(PlanetaryPosition {
        planet: Planet::Saturn,
        longitude: sat_lon_deg,
        sign: ZodiacSign::from_longitude(sat_lon_deg, use_13_signs),
        retrograde: is_retrograde(&planet::Planet::Saturn, jd),
        moon_phase: None,
    });

    // Moon - geocentric ecliptic position (never retrograde)
    // Calculate moon phase from Sun-Moon angular separation
    let (moon_ecl, _) = lunar::geocent_ecl_pos(jd);
    let moon_lon_deg = angle::limit_to_360(moon_ecl.long.to_degrees());
    let sun_moon_angle = (moon_lon_deg - sun_lon_deg).rem_euclid(360.0);
    let phase = MoonPhase::from_angle(sun_moon_angle);

    positions.push(PlanetaryPosition {
        planet: Planet::Moon,
        longitude: moon_lon_deg,
        sign: ZodiacSign::from_longitude(moon_lon_deg, use_13_signs),
        retrograde: false,
        moon_phase: Some(phase),
    });

    positions
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_zodiac_from_longitude_traditional() {
        // Test traditional 12-sign zodiac (equal 30° divisions)
        assert_eq!(ZodiacSign::from_longitude(0.0, false), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_longitude(30.0, false), ZodiacSign::Taurus);
        assert_eq!(ZodiacSign::from_longitude(60.0, false), ZodiacSign::Gemini);
        assert_eq!(ZodiacSign::from_longitude(210.0, false), ZodiacSign::Scorpio);
        assert_eq!(ZodiacSign::from_longitude(240.0, false), ZodiacSign::Sagittarius);
        assert_eq!(ZodiacSign::from_longitude(330.0, false), ZodiacSign::Pisces);
        assert_eq!(ZodiacSign::from_longitude(360.0, false), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_longitude(390.0, false), ZodiacSign::Taurus);
    }

    #[test]
    fn test_zodiac_from_longitude_13_signs() {
        // Test IAU constellation boundaries (13-sign zodiac with Ophiuchus)
        assert_eq!(ZodiacSign::from_longitude(0.0, true), ZodiacSign::Pisces);    // 0° is in Pisces
        assert_eq!(ZodiacSign::from_longitude(29.0, true), ZodiacSign::Aries);    // Aries starts at 29°
        assert_eq!(ZodiacSign::from_longitude(40.0, true), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_longitude(53.5, true), ZodiacSign::Taurus);   // Taurus starts at 53.5°
        assert_eq!(ZodiacSign::from_longitude(100.0, true), ZodiacSign::Gemini);
        assert_eq!(ZodiacSign::from_longitude(241.0, true), ZodiacSign::Scorpio); // Scorpio starts at 241°
        assert_eq!(ZodiacSign::from_longitude(248.0, true), ZodiacSign::Ophiuchus); // Ophiuchus starts at 248°
        assert_eq!(ZodiacSign::from_longitude(255.0, true), ZodiacSign::Ophiuchus);
        assert_eq!(ZodiacSign::from_longitude(266.0, true), ZodiacSign::Sagittarius); // Sagittarius at 266°
        assert_eq!(ZodiacSign::from_longitude(351.5, true), ZodiacSign::Pisces);  // Pisces starts at 351.5°
        assert_eq!(ZodiacSign::from_longitude(360.0, true), ZodiacSign::Pisces);  // Wraps to 0°
        assert_eq!(ZodiacSign::from_longitude(389.0, true), ZodiacSign::Aries);   // 389° = 29° (Aries)
    }

    #[test]
    fn test_zodiac_elements() {
        assert_eq!(ZodiacSign::Aries.element(), Element::Fire);
        assert_eq!(ZodiacSign::Taurus.element(), Element::Earth);
        assert_eq!(ZodiacSign::Gemini.element(), Element::Air);
        assert_eq!(ZodiacSign::Cancer.element(), Element::Water);
        assert_eq!(ZodiacSign::Ophiuchus.element(), Element::Fire); // Ophiuchus is Fire
    }

    #[test]
    fn test_astro_date_conversion() {
        // Test midnight - should be exactly day 1.0
        let dt_midnight = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let date_midnight = to_astro_date(&dt_midnight);
        assert_eq!(date_midnight.year, 2000);
        assert!(matches!(date_midnight.month, time::Month::Jan));
        assert_eq!(date_midnight.decimal_day, 1.0);

        // Test noon - should be day 1.5
        let dt_noon = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let date_noon = to_astro_date(&dt_noon);
        assert_eq!(date_noon.decimal_day, 1.5);

        // Test 6:30:30 - should be 1 + 6/24 + 30/1440 + 30/86400
        let dt_morning = Utc.with_ymd_and_hms(2000, 1, 1, 6, 30, 30).unwrap();
        let date_morning = to_astro_date(&dt_morning);
        let expected = 1.0 + 6.0 / 24.0 + 30.0 / 1440.0 + 30.0 / 86400.0;
        assert!((date_morning.decimal_day - expected).abs() < 0.0001);
    }

    #[test]
    fn test_planetary_positions() {
        let test_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let positions = calculate_planetary_positions(test_time);

        assert_eq!(positions.len(), 7);

        let planet_names: Vec<_> = positions.iter().map(|p| p.planet).collect();
        assert!(planet_names.contains(&Planet::Sun));
        assert!(planet_names.contains(&Planet::Moon));
        assert!(planet_names.contains(&Planet::Mercury));
        assert!(planet_names.contains(&Planet::Venus));
        assert!(planet_names.contains(&Planet::Mars));
        assert!(planet_names.contains(&Planet::Jupiter));
        assert!(planet_names.contains(&Planet::Saturn));

        for pos in &positions {
            assert!(pos.longitude >= 0.0 && pos.longitude < 360.0,
                    "Planet {:?} longitude {} out of range", pos.planet, pos.longitude);
        }
    }


    #[test]
    fn test_planet_domains() {
        assert_eq!(Planet::Mercury.domain(), "Communication & Network");
        assert_eq!(Planet::Mars.domain(), "Energy & CPU-Intensive");
        assert_eq!(Planet::Venus.domain(), "Harmony & Desktop/UI");
    }

    #[test]
    fn test_november_2025_positions_traditional() {
        let test_time = Utc.with_ymd_and_hms(2025, 11, 19, 22, 7, 46).unwrap();
        let positions = calculate_planetary_positions(test_time);

        // Expected positions from MoonTracks ephemeris (traditional tropical zodiac):
        // Sun: 26°54' Scorpio (210° + 26.9° = ~236.9°)
        // Mercury: 29°11' Scorpio (210° + 29.2° = ~239.2°)
        // Venus: 15°07' Scorpio (210° + 15.12° = ~225.1°)
        // Mars: 10°28' Sagittarius (240° + 10.47° = ~250.5°)
        // Jupiter: 25°04' Cancer (90° + 25.07° = ~115.1°)
        // Saturn: 25°14' Pisces (330° + 25.23° = ~355.2°)
        // Moon: 13°00' Scorpio (210° + 13° = ~223°)
        for pos in &positions {
            println!("{:?} at {:.1}° in {:?}", pos.planet, pos.longitude, pos.sign);
            match pos.planet {
                Planet::Sun => {
                    assert_eq!(pos.sign, ZodiacSign::Scorpio, "Sun should be in Scorpio");
                    assert!(pos.longitude >= 210.0 && pos.longitude < 240.0, "Sun longitude out of expected range");
                }
                Planet::Mercury => {
                    assert_eq!(pos.sign, ZodiacSign::Scorpio, "Mercury should be in Scorpio");
                }
                Planet::Venus => {
                    assert_eq!(pos.sign, ZodiacSign::Scorpio, "Venus should be in Scorpio");
                }
                Planet::Mars => {
                    assert_eq!(pos.sign, ZodiacSign::Sagittarius, "Mars should be in Sagittarius");
                }
                Planet::Jupiter => {
                    assert_eq!(pos.sign, ZodiacSign::Cancer, "Jupiter should be in Cancer");
                }
                Planet::Saturn => {
                    assert_eq!(pos.sign, ZodiacSign::Pisces, "Saturn should be in Pisces");
                }
                Planet::Moon => {
                    assert_eq!(pos.sign, ZodiacSign::Scorpio, "Moon should be in Scorpio");
                }
            }
        }
    }

    #[test]
    fn test_november_2025_positions_13_signs() {
        let test_time = Utc.with_ymd_and_hms(2025, 11, 19, 22, 7, 46).unwrap();
        let positions = calculate_planetary_positions_with_zodiac(test_time, true);

        // Same date but with IAU 13-sign boundaries:
        // Many "Scorpio" positions fall in Libra because IAU Scorpio is only ~7° wide (241-248°)
        // Mars at ~250.5° is in Ophiuchus (248-266°)
        for pos in &positions {
            println!("{:?} at {:.1}° in {:?} (13-sign)", pos.planet, pos.longitude, pos.sign);
            match pos.planet {
                Planet::Sun => {
                    assert_eq!(pos.sign, ZodiacSign::Libra, "Sun should be in Libra (IAU)");
                }
                Planet::Mercury => {
                    assert_eq!(pos.sign, ZodiacSign::Libra, "Mercury should be in Libra (IAU)");
                }
                Planet::Venus => {
                    assert_eq!(pos.sign, ZodiacSign::Libra, "Venus should be in Libra (IAU)");
                }
                Planet::Mars => {
                    assert_eq!(pos.sign, ZodiacSign::Ophiuchus, "Mars should be in Ophiuchus (IAU)");
                }
                Planet::Jupiter => {
                    assert_eq!(pos.sign, ZodiacSign::Gemini, "Jupiter should be in Gemini (IAU)");
                }
                Planet::Saturn => {
                    assert_eq!(pos.sign, ZodiacSign::Pisces, "Saturn should be in Pisces (IAU)");
                }
                Planet::Moon => {
                    assert_eq!(pos.sign, ZodiacSign::Libra, "Moon should be in Libra (IAU)");
                }
            }
        }
    }
}
