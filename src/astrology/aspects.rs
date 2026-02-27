use super::planets::{Planet, PlanetaryPosition};

/// The classical astrological aspects (angular relationships between planets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AspectType {
    Conjunction,  // 0°   — planets merge energies
    Opposition,   // 180° — planets in tension
    Trine,        // 120° — harmonious flow
    Square,       // 90°  — dynamic challenge
    Sextile,      // 60°  — opportunity
    Quincunx,     // 150° — adjustment required
}

/// Whether an aspect is beneficial, challenging, or neutral for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectNature {
    Harmonious,
    Challenging,
    Neutral,
}

/// A detected angular relationship between two planets
#[derive(Debug, Clone)]
pub struct Aspect {
    pub planet_a: Planet,
    pub planet_b: Planet,
    pub aspect_type: AspectType,
    pub nature: AspectNature,
    pub orb: f64,       // degrees off from the exact aspect angle
    pub strength: f64,  // 1.0 = exact, approaches 0.0 at the edge of the orb
}

impl AspectType {
    pub fn target_angle(self) -> f64 {
        match self {
            AspectType::Conjunction => 0.0,
            AspectType::Opposition  => 180.0,
            AspectType::Trine       => 120.0,
            AspectType::Square      => 90.0,
            AspectType::Sextile     => 60.0,
            AspectType::Quincunx    => 150.0,
        }
    }

    pub fn max_orb(self) -> f64 {
        match self {
            AspectType::Conjunction |
            AspectType::Opposition  |
            AspectType::Trine       |
            AspectType::Square      => 8.0,
            AspectType::Sextile     => 6.0,
            AspectType::Quincunx    => 3.0,
        }
    }

    // Conjunction is context-dependent — treated Neutral
    pub fn nature(self) -> AspectNature {
        match self {
            AspectType::Trine | AspectType::Sextile                  => AspectNature::Harmonious,
            AspectType::Square | AspectType::Opposition |
            AspectType::Quincunx                                     => AspectNature::Challenging,
            AspectType::Conjunction                                   => AspectNature::Neutral,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            AspectType::Conjunction => "Conjunction",
            AspectType::Opposition  => "Opposition",
            AspectType::Trine       => "Trine",
            AspectType::Square      => "Square",
            AspectType::Sextile     => "Sextile",
            AspectType::Quincunx    => "Quincunx",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            AspectType::Conjunction => "☌",
            AspectType::Opposition  => "☍",
            AspectType::Trine       => "△",
            AspectType::Square      => "□",
            AspectType::Sextile     => "⚹",
            AspectType::Quincunx    => "⚻",
        }
    }

    /// Scheduling priority multiplier when this aspect involves a task's ruling planet
    pub fn scheduling_modifier(self) -> f64 {
        match self {
            AspectType::Trine       => 1.4,
            AspectType::Conjunction => 1.3,
            AspectType::Sextile     => 1.2,
            AspectType::Quincunx    => 0.8,
            AspectType::Square      => 0.7,
            AspectType::Opposition  => 0.6,
        }
    }
}

/// Compute the smallest angular separation between two ecliptic longitudes (0–180°)
fn angular_separation(lon_a: f64, lon_b: f64) -> f64 {
    let diff = (lon_b - lon_a).rem_euclid(360.0);
    if diff <= 180.0 { diff } else { 360.0 - diff }
}

/// Detect all aspects between all pairs of planets in `positions`.
/// When a separation matches multiple aspect types, only the closest match is kept.
pub fn calculate_aspects(positions: &[PlanetaryPosition]) -> Vec<Aspect> {
    let all_types = [
        AspectType::Conjunction,
        AspectType::Opposition,
        AspectType::Trine,
        AspectType::Square,
        AspectType::Sextile,
        AspectType::Quincunx,
    ];

    let mut aspects = Vec::new();

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let pa = &positions[i];
            let pb = &positions[j];

            let sep = angular_separation(pa.longitude, pb.longitude);

            // Find the tightest matching aspect type
            let best = all_types.iter().filter_map(|&asp| {
                let orb = (sep - asp.target_angle()).abs();
                if orb <= asp.max_orb() {
                    Some((asp, orb))
                } else {
                    None
                }
            }).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((asp_type, orb)) = best {
                let strength = 1.0 - (orb / asp_type.max_orb());
                aspects.push(Aspect {
                    planet_a:    pa.planet,
                    planet_b:    pb.planet,
                    aspect_type: asp_type,
                    nature:      asp_type.nature(),
                    orb,
                    strength,
                });
            }
        }
    }

    aspects
}

/// Return all aspects that involve `planet` (either as planet_a or planet_b)
pub fn aspects_for_planet<'a>(aspects: &'a [Aspect], planet: Planet) -> Vec<&'a Aspect> {
    aspects.iter()
        .filter(|a| a.planet_a == planet || a.planet_b == planet)
        .collect()
}

/// Combine all aspect modifiers for a planet into a single scheduling multiplier.
/// Harmonious aspects stack multiplicatively as boosts; challenging ones as penalties.
/// Result is clamped to [0.3, 2.0].
pub fn combined_aspect_modifier(aspects: &[Aspect], planet: Planet) -> f64 {
    let relevant = aspects_for_planet(aspects, planet);
    if relevant.is_empty() {
        return 1.0;
    }

    let modifier = relevant.iter().fold(1.0_f64, |acc, asp| {
        // Weight the modifier by the aspect's strength (exact = full effect)
        let raw = asp.aspect_type.scheduling_modifier();
        let weighted = 1.0 + (raw - 1.0) * asp.strength;
        acc * weighted
    });

    modifier.clamp(0.3, 2.0)
}

/// Generate human-readable flavor text describing how aspects affect a planet's domain
pub fn describe_aspects(aspects: &[Aspect], planet: Planet) -> String {
    let relevant = aspects_for_planet(aspects, planet);

    if relevant.is_empty() {
        return format!(
            "{} stands alone in the cosmos — unaspected, operating on pure instinct",
            planet.name()
        );
    }

    let mut parts: Vec<String> = Vec::new();

    for asp in &relevant {
        let other = if asp.planet_a == planet { asp.planet_b } else { asp.planet_a };
        let (emoji, flavor) = match asp.aspect_type {
            AspectType::Trine => (
                "✨",
                format!("{} {} {} {} — cosmic harmony flows freely!", planet.name(), asp.aspect_type.symbol(), other.name(), asp.aspect_type.name()),
            ),
            AspectType::Sextile => (
                "🌟",
                format!("{} {} {} {} — opportunity knocks from the stars!", planet.name(), asp.aspect_type.symbol(), other.name(), asp.aspect_type.name()),
            ),
            AspectType::Conjunction => (
                "☄️",
                format!("{} {} {} {} — energies merge in cosmic union!", planet.name(), asp.aspect_type.symbol(), other.name(), asp.aspect_type.name()),
            ),
            AspectType::Opposition => (
                "⚠️",
                format!("{} {} {} {} — cosmic tension! Opposing forces clash!", planet.name(), asp.aspect_type.symbol(), other.name(), asp.aspect_type.name()),
            ),
            AspectType::Square => (
                "⚔️",
                format!("{} {} {} {} — dynamic friction creates challenges!", planet.name(), asp.aspect_type.symbol(), other.name(), asp.aspect_type.name()),
            ),
            AspectType::Quincunx => (
                "🌀",
                format!("{} {} {} {} — adjustment needed, stars misaligned!", planet.name(), asp.aspect_type.symbol(), other.name(), asp.aspect_type.name()),
            ),
        };
        parts.push(format!("{} {}", emoji, flavor));
    }

    parts.join(" | ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::planets::{ZodiacSign, MoonPhase};

    fn make_pos(planet: Planet, longitude: f64) -> PlanetaryPosition {
        PlanetaryPosition {
            planet,
            longitude,
            sign: ZodiacSign::from_longitude(longitude, false),
            retrograde: false,
            moon_phase: if planet == Planet::Moon { Some(MoonPhase::from_angle(longitude)) } else { None },
        }
    }

    #[test]
    fn test_opposition_detection() {
        // Exact opposition
        let aspects = calculate_aspects(&[make_pos(Planet::Sun, 0.0), make_pos(Planet::Moon, 180.0)]);
        assert_eq!(aspects.len(), 1);
        assert_eq!(aspects[0].aspect_type, AspectType::Opposition);
        assert!((aspects[0].orb).abs() < 0.01);

        // Within orb (5° off)
        let aspects = calculate_aspects(&[make_pos(Planet::Sun, 0.0), make_pos(Planet::Moon, 175.0)]);
        assert_eq!(aspects[0].aspect_type, AspectType::Opposition);

        // Outside orb (10° off — greater than 8°)
        let aspects = calculate_aspects(&[make_pos(Planet::Sun, 0.0), make_pos(Planet::Moon, 170.0)]);
        assert!(aspects.iter().all(|a| a.aspect_type != AspectType::Opposition));
    }

    #[test]
    fn test_trine_detection() {
        // Exact trine
        let aspects = calculate_aspects(&[make_pos(Planet::Mars, 0.0), make_pos(Planet::Jupiter, 120.0)]);
        assert_eq!(aspects[0].aspect_type, AspectType::Trine);

        // Trine wrapping around 0°: 350° and 110° → 120° separation
        let aspects = calculate_aspects(&[make_pos(Planet::Saturn, 350.0), make_pos(Planet::Mercury, 110.0)]);
        assert!(aspects.iter().any(|a| a.aspect_type == AspectType::Trine), "Expected trine across 0°");
    }

    #[test]
    fn test_conjunction_detection() {
        // Exact conjunction
        let aspects = calculate_aspects(&[make_pos(Planet::Venus, 45.0), make_pos(Planet::Mars, 45.0)]);
        assert_eq!(aspects[0].aspect_type, AspectType::Conjunction);

        // Conjunction across 0°: 358° and 2° → 4° separation
        let aspects = calculate_aspects(&[make_pos(Planet::Sun, 358.0), make_pos(Planet::Moon, 2.0)]);
        assert!(aspects.iter().any(|a| a.aspect_type == AspectType::Conjunction), "Expected conjunction across 0°");
    }

    #[test]
    fn test_no_aspects_edge_cases() {
        assert!(calculate_aspects(&[]).is_empty());
        assert!(calculate_aspects(&[make_pos(Planet::Sun, 90.0)]).is_empty());
    }

    #[test]
    fn test_combined_modifier_clamped() {
        // Stack many aspects — result must stay within [0.3, 2.0]
        let positions = vec![
            make_pos(Planet::Sun,     0.0),
            make_pos(Planet::Moon,    120.0), // Trine
            make_pos(Planet::Mercury, 60.0),  // Sextile
            make_pos(Planet::Mars,    180.0), // Opposition
            make_pos(Planet::Jupiter, 90.0),  // Square
        ];
        let aspects = calculate_aspects(&positions);
        let modifier = combined_aspect_modifier(&aspects, Planet::Sun);
        assert!(modifier >= 0.3 && modifier <= 2.0, "modifier {modifier} out of range");
        assert!((combined_aspect_modifier(&[], Planet::Sun) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_aspect_natures() {
        assert_eq!(AspectType::Trine.nature(), AspectNature::Harmonious);
        assert_eq!(AspectType::Sextile.nature(), AspectNature::Harmonious);
        assert_eq!(AspectType::Square.nature(), AspectNature::Challenging);
        assert_eq!(AspectType::Opposition.nature(), AspectNature::Challenging);
        assert_eq!(AspectType::Quincunx.nature(), AspectNature::Challenging);
        assert_eq!(AspectType::Conjunction.nature(), AspectNature::Neutral);
    }

    #[test]
    fn test_describe_unaspected() {
        let desc = describe_aspects(&[], Planet::Mercury);
        assert!(desc.contains("unaspected") || desc.contains("alone"));
    }
}
