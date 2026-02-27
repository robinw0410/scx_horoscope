use std::collections::HashMap;
use std::path::Path;
use chrono::{DateTime, Duration, Utc};

use super::planets::{
    calculate_planetary_positions_with_zodiac, Element, MoonPhase, Planet,
    PlanetaryPosition, ZodiacSign,
};
use super::tasks::TaskType;

// ─────────────────────────────────────────────────────────────────────────────
// ProcessBirthChart
// ─────────────────────────────────────────────────────────────────────────────

/// The natal (birth) astrological chart for a running process.
#[derive(Debug, Clone)]
pub struct ProcessBirthChart {
    pub pid: i32,
    pub birth_time: DateTime<Utc>,
    pub natal_positions: Vec<PlanetaryPosition>,
    pub sun_sign: ZodiacSign,
    pub ascendant_element: Element,
    pub natal_moon_phase: MoonPhase,
}

impl ProcessBirthChart {
    /// Build a birth chart by reading `/proc/<pid>/stat`.
    /// Returns `None` if the process does not exist or its start time cannot
    /// be determined.
    pub fn from_pid(pid: i32, use_13_signs: bool) -> Option<Self> {
        let birth_time = read_process_birth_time(pid)?;
        let natal_positions =
            calculate_planetary_positions_with_zodiac(birth_time, use_13_signs);

        let sun_sign = natal_positions
            .iter()
            .find(|p| p.planet == Planet::Sun)
            .map(|p| p.sign)?;

        let ascendant_element = sun_sign.element();

        let natal_moon_phase = natal_positions
            .iter()
            .find(|p| p.planet == Planet::Moon)
            .and_then(|p| p.moon_phase)
            .unwrap_or(MoonPhase::NewMoon);

        Some(Self {
            pid,
            birth_time,
            natal_positions,
            sun_sign,
            ascendant_element,
            natal_moon_phase,
        })
    }

    pub fn natal_sign_for_planet(&self, planet: Planet) -> Option<ZodiacSign> {
        self.natal_positions
            .iter()
            .find(|p| p.planet == planet)
            .map(|p| p.sign)
    }

    /// Compare natal chart against current planetary positions for the ruling
    /// planet of the given task type. Returns a scheduling multiplier:
    /// same element = 1.3, compatible = 1.1, neutral = 1.0, opposing = 0.8
    pub fn compatibility_with_current(
        &self,
        current_positions: &[PlanetaryPosition],
        task_type: TaskType,
    ) -> f64 {
        let ruling_planet = task_type.ruling_planet();

        let natal_element = self
            .natal_positions
            .iter()
            .find(|p| p.planet == ruling_planet)
            .map(|p| p.sign.element());

        let current_element = current_positions
            .iter()
            .find(|p| p.planet == ruling_planet)
            .map(|p| p.sign.element());

        match (natal_element, current_element) {
            (Some(natal), Some(current)) => element_compatibility(natal, current),
            _ => 1.0,
        }
    }

    pub fn describe_natal_chart(&self) -> String {
        let destiny = match self.ascendant_element {
            Element::Fire  => "blazing computational glory",
            Element::Earth => "steadfast, long-running endurance",
            Element::Air   => "swift network communication",
            Element::Water => "deep memory exploration",
        };

        format!(
            "🌟 Born under {} with Moon in {}! A {} spirit destined for {}!",
            self.sun_sign.name(),
            self.natal_moon_phase.name(),
            self.ascendant_element.name(),
            destiny,
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ProcessRegistry
// ─────────────────────────────────────────────────────────────────────────────

/// A cache of birth charts for currently running processes.
pub struct ProcessRegistry {
    charts: HashMap<i32, ProcessBirthChart>,
    use_13_signs: bool,
}

impl ProcessRegistry {
    pub fn new(use_13_signs: bool) -> Self {
        Self {
            charts: HashMap::new(),
            use_13_signs,
        }
    }

    /// Get or lazily create the birth chart for a process.
    pub fn get_or_create(&mut self, pid: i32) -> Option<&ProcessBirthChart> {
        if !self.charts.contains_key(&pid) {
            let chart = ProcessBirthChart::from_pid(pid, self.use_13_signs)?;
            self.charts.insert(pid, chart);
        }
        self.charts.get(&pid)
    }

    pub fn evict_dead_processes(&mut self) {
        self.charts
            .retain(|&pid, _| Path::new(&format!("/proc/{pid}")).exists());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

fn read_process_birth_time(pid: i32) -> Option<DateTime<Utc>> {
    let path = format!("/proc/{pid}/stat");
    let content = std::fs::read_to_string(&path).ok()?;

    // The comm field (field 2) can contain spaces and is wrapped in parentheses.
    // Find the closing ')' to safely skip it before splitting the rest.
    let after_comm = content.find(')').map(|i| &content[i + 1..])?;
    let fields: Vec<&str> = after_comm.split_whitespace().collect();

    // field 22 (1-indexed in /proc/pid/stat) = fields[19] in our after_comm slice
    let starttime: u64 = fields.get(19)?.parse().ok()?;

    let ticks_per_sec = procfs::ticks_per_second();
    let start_secs_since_boot = starttime / ticks_per_sec;
    let start_subsec_ticks   = starttime % ticks_per_sec;
    let start_nanos = (start_subsec_ticks * 1_000_000_000) / ticks_per_sec;

    let boot_time: DateTime<Utc> = procfs::boot_time().ok()?.into();

    let elapsed = Duration::seconds(start_secs_since_boot as i64)
        + Duration::nanoseconds(start_nanos as i64);

    Some(boot_time + elapsed)
}

fn element_compatibility(natal: Element, current: Element) -> f64 {
    if natal == current {
        return 1.3;
    }

    match (natal, current) {
        // Compatible pairs: Fire/Air (both active/upward) and Earth/Water (both receptive)
        (Element::Fire, Element::Air) | (Element::Air, Element::Fire)    => 1.1,
        (Element::Earth, Element::Water) | (Element::Water, Element::Earth) => 1.1,
        // Opposing pairs
        (Element::Fire, Element::Water) | (Element::Water, Element::Fire)   => 0.8,
        (Element::Earth, Element::Air)  | (Element::Air, Element::Earth)    => 0.8,
        _ => 1.0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_position(planet: Planet, sign: ZodiacSign) -> PlanetaryPosition {
        PlanetaryPosition {
            planet,
            longitude: 0.0,
            sign,
            retrograde: false,
            moon_phase: if planet == Planet::Moon {
                Some(MoonPhase::FullMoon)
            } else {
                None
            },
        }
    }

    fn full_natal(mars_sign: ZodiacSign) -> Vec<PlanetaryPosition> {
        vec![
            make_position(Planet::Sun,     ZodiacSign::Leo),
            make_position(Planet::Moon,    ZodiacSign::Cancer),
            make_position(Planet::Mercury, ZodiacSign::Gemini),
            make_position(Planet::Venus,   ZodiacSign::Taurus),
            make_position(Planet::Mars,    mars_sign),
            make_position(Planet::Jupiter, ZodiacSign::Sagittarius),
            make_position(Planet::Saturn,  ZodiacSign::Capricorn),
        ]
    }

    #[test]
    fn test_registry_creation() {
        let registry = ProcessRegistry::new(false);
        assert!(!registry.use_13_signs);
        assert!(registry.charts.is_empty());

        let registry_13 = ProcessRegistry::new(true);
        assert!(registry_13.use_13_signs);
    }

    #[test]
    fn test_nonexistent_pid() {
        let result = ProcessBirthChart::from_pid(9_999_999, false);
        assert!(result.is_none(), "Expected None for non-existent PID");
    }

    #[test]
    fn test_compatibility_same_element() {
        // Mars in Aries (Fire) vs current Mars in Leo (Fire) → same element
        let chart = ProcessBirthChart {
            pid: 1,
            birth_time: Utc::now(),
            sun_sign: ZodiacSign::Leo,
            ascendant_element: Element::Fire,
            natal_moon_phase: MoonPhase::FullMoon,
            natal_positions: full_natal(ZodiacSign::Aries), // Mars = Fire
        };

        let current = vec![make_position(Planet::Mars, ZodiacSign::Leo)]; // Fire
        let m = chart.compatibility_with_current(&current, TaskType::CpuIntensive);
        assert!(m > 1.0, "Same element should yield > 1.0, got {m}");
    }

    #[test]
    fn test_compatibility_opposing_element() {
        // Mars in Aries (Fire) vs current Mars in Cancer (Water) → opposing
        let chart = ProcessBirthChart {
            pid: 2,
            birth_time: Utc::now(),
            sun_sign: ZodiacSign::Leo,
            ascendant_element: Element::Fire,
            natal_moon_phase: MoonPhase::NewMoon,
            natal_positions: full_natal(ZodiacSign::Aries), // Mars = Fire
        };

        let current = vec![make_position(Planet::Mars, ZodiacSign::Cancer)]; // Water opposes Fire
        let m = chart.compatibility_with_current(&current, TaskType::CpuIntensive);
        assert!(m < 1.0, "Opposing elements should yield < 1.0, got {m}");
    }
}
