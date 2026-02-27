use super::planets::{Planet, Element, PlanetaryPosition, MoonPhase,
                     calculate_planetary_positions_with_zodiac};
#[cfg(test)]
use super::planets::calculate_planetary_positions;
use super::tasks::{TaskType, TaskClassifier};
use super::aspects::{calculate_aspects, combined_aspect_modifier, describe_aspects};
use super::birth_chart::ProcessRegistry;
use super::predictions::{HoroscopePrediction, get_system_daily_horoscope};
use chrono::{DateTime, Utc};

/// Scheduling decision with astrological reasoning
#[derive(Debug, Clone)]
pub struct SchedulingDecision {
    pub priority: u32,
    pub reasoning: String,
    pub planetary_influence: f64,  // -1.0 to 1.0
    #[allow(dead_code)]  // Used internally in calculations, not accessed externally
    pub element_boost: f64,         // Multiplier (includes moon phase for Interactive tasks)
}

/// The main astrological scheduler
pub struct AstrologicalScheduler {
    classifier: TaskClassifier,
    planetary_cache: Option<(DateTime<Utc>, Vec<PlanetaryPosition>)>,
    cache_duration_secs: i64,
    use_13_signs: bool,       // Use 13-sign zodiac with Ophiuchus (IAU boundaries)
    birth_registry: ProcessRegistry,  // Natal charts for running processes
    enable_birth_charts: bool,
    enable_aspects: bool,
    enable_predictions: bool,
    enable_cpu_affinity: bool,
}

/// Which CPU element is "attuned" to a given CPU index (0-based)
/// Cycle: Fire, Earth, Air, Water — groups of 3 CPUs each
fn cpu_element(cpu_index: i32) -> Element {
    match cpu_index.rem_euclid(4) {
        0 => Element::Fire,
        1 => Element::Earth,
        2 => Element::Air,
        _ => Element::Water,
    }
}

/// Astrological compatibility between a task's element and a CPU's element.
/// Returns a modifier to apply when computing CPU affinity preference.
fn cpu_task_affinity(cpu_element: Element, task_element: Element) -> f64 {
    if cpu_element == task_element {
        return 1.5; // Perfect match — this CPU was born for this task
    }
    match (cpu_element, task_element) {
        // Compatible pairs
        (Element::Fire, Element::Air)   | (Element::Air, Element::Fire)   => 1.2,
        (Element::Earth, Element::Water)| (Element::Water, Element::Earth) => 1.2,
        // Opposing pairs
        (Element::Fire, Element::Water) | (Element::Water, Element::Fire)  => 0.6,
        (Element::Earth, Element::Air)  | (Element::Air, Element::Earth)   => 0.6,
        _ => 1.0,
    }
}

/// Map element to a simple zodiac sign for display in cosmic weather
fn element_sign(element: Element) -> &'static str {
    match element {
        Element::Fire  => "Aries ♈",
        Element::Earth => "Taurus ♉",
        Element::Air   => "Gemini ♊",
        Element::Water => "Cancer ♋",
    }
}

impl AstrologicalScheduler {
    pub fn new(cache_duration_secs: i64) -> Self {
        Self::with_options(cache_duration_secs, false)
    }

    pub fn with_options(cache_duration_secs: i64, use_13_signs: bool) -> Self {
        Self::with_full_options(cache_duration_secs, use_13_signs, false, false, false, false)
    }

    /// Create scheduler with all feature flags
    pub fn with_full_options(
        cache_duration_secs: i64,
        use_13_signs: bool,
        enable_birth_charts: bool,
        enable_aspects: bool,
        enable_predictions: bool,
        enable_cpu_affinity: bool,
    ) -> Self {
        Self {
            classifier: TaskClassifier::new(),
            planetary_cache: None,
            cache_duration_secs,
            use_13_signs,
            birth_registry: ProcessRegistry::new(use_13_signs),
            enable_birth_charts,
            enable_aspects,
            enable_predictions,
            enable_cpu_affinity,
        }
    }

    /// Returns whether the 13-sign zodiac (with Ophiuchus) is enabled
    #[allow(dead_code)]  // Public API
    pub fn uses_13_signs(&self) -> bool {
        self.use_13_signs
    }

    /// Get a birth-chart compatibility modifier for a process
    pub fn birth_chart_modifier(&mut self, pid: i32, current_positions: &[PlanetaryPosition], task_type: TaskType) -> f64 {
        if !self.enable_birth_charts {
            return 1.0;
        }
        self.birth_registry
            .get_or_create(pid)
            .map(|chart| chart.compatibility_with_current(current_positions, task_type))
            .unwrap_or(1.0)
    }

    /// Pick the most astrologically compatible CPU for a task
    /// Returns an optional CPU preference score (higher = better match)
    pub fn cpu_affinity_score(&self, cpu_index: i32, task_type: TaskType, positions: &[PlanetaryPosition]) -> f64 {
        if !self.enable_cpu_affinity {
            return 1.0;
        }

        let cpu_elem = cpu_element(cpu_index);

        // Task element from ruling planet's current sign
        let ruling = task_type.ruling_planet();
        let task_elem = positions.iter()
            .find(|p| p.planet == ruling)
            .map(|p| p.sign.element())
            .unwrap_or(Element::Air);

        cpu_task_affinity(cpu_elem, task_elem)
    }

    /// Generate a horoscope prediction for a task (for debug logging)
    pub fn horoscope_prediction(&self, positions: &[PlanetaryPosition], task_type: TaskType, now: DateTime<Utc>) -> Option<HoroscopePrediction> {
        if self.enable_predictions {
            Some(HoroscopePrediction::generate(positions, task_type, &now))
        } else {
            None
        }
    }

    /// Periodically evict dead processes from the birth chart registry
    pub fn evict_dead_processes(&mut self) {
        if self.enable_birth_charts {
            self.birth_registry.evict_dead_processes();
        }
    }

    /// Classify a task by command name (public wrapper for main.rs use)
    pub fn classify_for_affinity(&self, comm: &str) -> TaskType {
        self.classifier.classify(comm)
    }

    /// Return a snapshot of the current cached planetary positions
    pub fn current_positions(&mut self, now: DateTime<Utc>) -> Vec<PlanetaryPosition> {
        self.get_planetary_positions(now).clone()
    }

    fn get_planetary_positions(&mut self, now: DateTime<Utc>) -> &Vec<PlanetaryPosition> {
        let needs_refresh = match &self.planetary_cache {
            None => true,
            Some((cached_time, _)) => {
                now.timestamp() - cached_time.timestamp() > self.cache_duration_secs
            }
        };

        if needs_refresh {
            let positions = calculate_planetary_positions_with_zodiac(now, self.use_13_signs);
            self.planetary_cache = Some((now, positions));
        }

        &self.planetary_cache.as_ref().unwrap().1
    }

    fn calculate_planetary_influence(position: &PlanetaryPosition) -> f64 {
        // Retrograde planets have NEGATIVE influence (causes time slice penalty)
        if position.retrograde {
            return -1.0;
        }

        // Apply element modifier to planetary influence when direct
        match position.sign.element() {
            Element::Fire => 1.3,
            Element::Earth => 1.1,
            Element::Air => 1.2,
            Element::Water => 1.0,
        }
    }

    fn moon_phase_modifier(phase: MoonPhase) -> f64 {
        match phase {
            // Full Moon - peak emotional/interactive energy
            MoonPhase::FullMoon => 1.4,
            // Waxing phases - growing energy
            MoonPhase::WaxingGibbous => 1.2,
            MoonPhase::FirstQuarter => 1.1,
            MoonPhase::WaxingCrescent => 1.05,
            // New Moon - minimal energy
            MoonPhase::NewMoon => 0.8,
            // Waning phases - declining energy
            MoonPhase::WaningGibbous => 0.95,
            MoonPhase::LastQuarter => 0.9,
            MoonPhase::WaningCrescent => 0.85,
        }
    }

    /// Moon phase modifier scaled for memory/I-O tasks (subtler tidal effect than Interactive)
    fn moon_phase_tidal_modifier(phase: MoonPhase) -> f64 {
        match phase {
            MoonPhase::FullMoon => 1.2,         // High tide: memory pools swell
            MoonPhase::WaxingGibbous => 1.1,
            MoonPhase::FirstQuarter => 1.05,
            MoonPhase::WaxingCrescent => 1.02,
            MoonPhase::NewMoon => 0.9,          // Low tide: memory ebbs
            MoonPhase::WaningGibbous => 0.97,
            MoonPhase::LastQuarter => 0.95,
            MoonPhase::WaningCrescent => 0.92,
        }
    }

    /// Moon phase modifier for network tasks (Mercury-ruled but flows like tides)
    fn moon_phase_flow_modifier(phase: MoonPhase) -> f64 {
        match phase {
            MoonPhase::FullMoon => 1.15,        // Maximum cosmic bandwidth
            MoonPhase::WaxingGibbous => 1.08,
            MoonPhase::FirstQuarter => 1.04,
            MoonPhase::WaxingCrescent => 1.01,
            MoonPhase::NewMoon => 0.93,         // Dark moon = dark packets
            MoonPhase::WaningGibbous => 0.98,
            MoonPhase::LastQuarter => 0.96,
            MoonPhase::WaningCrescent => 0.94,
        }
    }

    fn calculate_element_boost(positions: &[PlanetaryPosition], task_type: TaskType) -> f64 {
        let ruling_planet = task_type.ruling_planet();

        let planet_pos = positions.iter()
            .find(|p| p.planet == ruling_planet)
            .expect("Ruling planet should always be present");

        let element = planet_pos.sign.element();

        // Strong boost for matching elements, DEBUFF for opposing elements!
        match (element, task_type) {
            // Perfect matches - BOOSTED
            (Element::Fire, TaskType::CpuIntensive) | (Element::Air, TaskType::Network) => 1.5,
            (Element::Earth, TaskType::System) => 1.4,
            (Element::Water, TaskType::MemoryHeavy) | (Element::Air | Element::Water, TaskType::Desktop) => 1.3,

            // Opposing elements - DEBUFFED (Fire opposes Water, Earth opposes Air)
            (Element::Water, TaskType::CpuIntensive) | (Element::Earth, TaskType::Network) => 0.6,
            (Element::Air, TaskType::System) | (Element::Fire, TaskType::MemoryHeavy) => 0.7,

            // Neutral combinations
            _ => 1.0,
        }
    }

    pub fn schedule_task(
        &mut self,
        comm: &str,
        pid: i32,
        now: DateTime<Utc>,
    ) -> SchedulingDecision {
        if TaskClassifier::is_critical(pid) {
            return SchedulingDecision {
                priority: 1000,
                reasoning: format!("☀️ Sun rules all - PID {pid} is CRITICAL (init)"),
                planetary_influence: 1.0,
                element_boost: 2.0,
            };
        }

        let task_type = self.classifier.classify(comm);
        let ruling_planet = task_type.ruling_planet();

        // Clone positions immediately so we own the data — this avoids borrow
        // conflicts when later calling &mut self methods (birth_chart_modifier).
        let positions: Vec<PlanetaryPosition> = self.get_planetary_positions(now).clone();

        let planet_pos = positions.iter()
            .find(|p| p.planet == ruling_planet)
            .expect("Ruling planet should always be present");

        let planetary_influence = Self::calculate_planetary_influence(planet_pos);
        let mut element_boost = Self::calculate_element_boost(&positions, task_type);

        // ── Moon phase effects (extended to Memory and Network tasks) ──────────
        if let Some(moon_pos) = positions.iter().find(|p| p.planet == Planet::Moon) {
            if let Some(phase) = moon_pos.moon_phase {
                match task_type {
                    // Interactive tasks: full moon phase modifier (original)
                    TaskType::Interactive => {
                        element_boost *= Self::moon_phase_modifier(phase);
                    }
                    // Memory tasks: tidal moon effect on memory pools
                    TaskType::MemoryHeavy => {
                        element_boost *= Self::moon_phase_tidal_modifier(phase);
                    }
                    // Network tasks: moon governs data tides too
                    TaskType::Network => {
                        element_boost *= Self::moon_phase_flow_modifier(phase);
                    }
                    _ => {}
                }
            }
        }

        // ── Planetary aspects: boost or penalise based on cosmic alignments ───
        let enable_aspects = self.enable_aspects;
        let aspect_modifier = if enable_aspects {
            let aspects = calculate_aspects(&positions);
            combined_aspect_modifier(&aspects, ruling_planet)
        } else {
            1.0
        };
        element_boost *= aspect_modifier;

        // ── Birth chart natal compatibility ───────────────────────────────────
        let birth_modifier = self.birth_chart_modifier(pid, &positions, task_type);
        element_boost *= birth_modifier;

        let base_priority = match task_type {
            TaskType::Critical => 1000,
            TaskType::System => 200,
            TaskType::Interactive => 150,
            TaskType::Desktop => 120,
            TaskType::CpuIntensive | TaskType::Network => 100,
            TaskType::MemoryHeavy => 80,
        };

        let influenced_priority = if planetary_influence >= 0.0 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let result = (f64::from(base_priority) * planetary_influence * element_boost) as u32;
            result
        } else {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let result = (f64::from(base_priority) * 0.3) as u32;
            result
        };

        let reasoning = Self::create_reasoning(
            task_type,
            planet_pos,
            planetary_influence,
            element_boost,
            aspect_modifier,
            birth_modifier,
        );

        SchedulingDecision {
            priority: influenced_priority.max(1),
            reasoning,
            planetary_influence,
            element_boost,
        }
    }

    fn create_reasoning(
        task_type: TaskType,
        planet_pos: &PlanetaryPosition,
        influence: f64,
        boost: f64,
        aspect_modifier: f64,
        birth_modifier: f64,
    ) -> String {
        let planet_name = planet_pos.planet.name();
        let sign_name = planet_pos.sign.name();
        let element_name = planet_pos.sign.element().name();

        // Retrograde takes precedence over all other conditions
        if influence < 0.0 {
            return format!(
                "☿℞ {} RETROGRADE in {} | {} task suffers cosmic CHAOS! Communications disrupted, delays expected",
                planet_name,
                sign_name,
                task_type.name()
            );
        }

        // Build aspect annotation
        let aspect_note = if (aspect_modifier - 1.0).abs() > 0.05 {
            if aspect_modifier > 1.0 {
                format!(" | △ Aspects: +{:.0}%", (aspect_modifier - 1.0) * 100.0)
            } else {
                format!(" | □ Aspects: -{:.0}%", (1.0 - aspect_modifier) * 100.0)
            }
        } else {
            String::new()
        };

        // Birth chart annotation
        let natal_note = if (birth_modifier - 1.0).abs() > 0.05 {
            if birth_modifier > 1.0 {
                format!(" | 🌟 Natal: +{:.0}%", (birth_modifier - 1.0) * 100.0)
            } else {
                format!(" | ☁️ Natal: -{:.0}%", (1.0 - birth_modifier) * 100.0)
            }
        } else {
            String::new()
        };

        if boost < 0.7 {
            // DEBUFFED! Opposing elements clash
            let opposition = match (planet_pos.sign.element(), task_type) {
                (Element::Water, TaskType::CpuIntensive) => "💧 Water dampens fire",
                (Element::Earth, TaskType::Network) => "🪨 Earth blocks air",
                (Element::Air, TaskType::System) => "💨 Air disrupts earth",
                (Element::Fire, TaskType::MemoryHeavy) => "🔥 Fire evaporates water",
                _ => "⚔️ Elemental opposition",
            };
            format!(
                "⚠️ {} in {} ({}) | {} task DEBUFFED | {}{}{}",
                planet_name, sign_name, element_name,
                task_type.name(), opposition, aspect_note, natal_note
            )
        } else if boost > 1.3 {
            format!(
                "✨ {} in {} ({}) | {} task COSMICALLY BLESSED | {} provides divine boost{}{}",
                planet_name, sign_name, element_name,
                task_type.name(), element_name, aspect_note, natal_note
            )
        } else if boost > 1.1 {
            format!(
                "{} in {} | {} task enhanced by favorable {} energy{}{}",
                planet_name, sign_name,
                task_type.name(), element_name, aspect_note, natal_note
            )
        } else {
            format!(
                "{} in {} | {} task neutral | Cosmos balanced{}{}",
                planet_name, sign_name,
                task_type.name(), aspect_note, natal_note
            )
        }
    }

    /// Get a full system daily horoscope using the predictions module
    pub fn get_daily_horoscope(&mut self, now: DateTime<Utc>) -> String {
        let positions = self.get_planetary_positions(now).clone();
        get_system_daily_horoscope(&positions, &now)
    }

    /// Get a summary of current astrological conditions
    pub fn get_cosmic_weather(&mut self, now: DateTime<Utc>) -> String {
        use std::fmt::Write;

        // Capture flags before the positions borrow (can't access self fields
        // while an &mut self borrow is alive via get_planetary_positions).
        let use_13_signs = self.use_13_signs;
        let enable_aspects = self.enable_aspects;
        let enable_cpu_affinity = self.enable_cpu_affinity;

        // Clone positions immediately so we own the data and there's no lingering
        // &mut self borrow preventing later field access.
        let positions: Vec<PlanetaryPosition> = self.get_planetary_positions(now).clone();

        let mut report = String::from("🌌 COSMIC WEATHER REPORT 🌌\n");
        let _ = writeln!(report, "Current time: {}", now.format("%Y-%m-%d %H:%M:%S UTC"));
        if use_13_signs {
            report.push_str("Zodiac system: 13-sign (IAU boundaries with Ophiuchus)\n");
        } else {
            report.push_str("Zodiac system: Traditional 12-sign (tropical)\n");
        }
        report.push('\n');

        for pos in &positions {
            let phase_info = if let Some(phase) = pos.moon_phase {
                format!(" [{}]", phase.name())
            } else {
                String::new()
            };
            #[allow(clippy::cast_possible_truncation)]
            let longitude_deg = pos.longitude as i32;
            let _ = writeln!(
                report,
                "{} in {} ({longitude_deg}°) - {}{}",
                pos.planet.name(),
                pos.sign.name(),
                pos.sign.element().name(),
                phase_info
            );
        }

        // Calculate element counts first for tension detection
        let elements: Vec<_> = positions.iter()
            .map(|p| p.sign.element())
            .collect();

        let fire_count = elements.iter().filter(|&&e| e == Element::Fire).count();
        let earth_count = elements.iter().filter(|&&e| e == Element::Earth).count();
        let air_count = elements.iter().filter(|&&e| e == Element::Air).count();
        let water_count = elements.iter().filter(|&&e| e == Element::Water).count();

        let fire_water_clash = fire_count >= 2 && water_count >= 2;
        let earth_air_clash = earth_count >= 2 && air_count >= 2;

        report.push_str("\n💫 ASTROLOGICAL GUIDANCE 💫\n\n");

        // Helper to generate status for each task type
        let task_status = |planet: Planet, ideal: Element, opposed: Element, clash: bool,
                          boosted_msg: &str, contested_msg: &str, debuffed_msg: &str| -> String {
            let pos = positions.iter().find(|p| p.planet == planet).unwrap();
            let element = pos.sign.element();
            match element {
                e if e == ideal && clash => format!("⚔️ BOOSTED but CONTESTED ⚔️ - {contested_msg}"),
                e if e == ideal => format!("✨ BOOSTED ✨ - {boosted_msg}"),
                e if e == opposed => format!("⚠️  DEBUFFED ⚠️  - {debuffed_msg}"),
                _ => "Neutral - Normal operations".to_string(),
            }
        };

        let mars_pos = positions.iter().find(|p| p.planet == Planet::Mars).unwrap();
        let _ = writeln!(report, "🔥 CPU-Intensive Tasks (Mars in {}): {}",
            mars_pos.sign.name(),
            task_status(Planet::Mars, Element::Fire, Element::Water, fire_water_clash,
                "Compilations and calculations favored!",
                "Fire powers CPU but Water planets oppose!",
                "Water dampens the CPU fires!"));

        let merc_pos = positions.iter().find(|p| p.planet == Planet::Mercury).unwrap();
        let _ = writeln!(report, "💬 Network Tasks (Mercury in {}): {}",
            merc_pos.sign.name(),
            task_status(Planet::Mercury, Element::Air, Element::Earth, earth_air_clash,
                "Network communications flow freely!",
                "Air speeds networks but Earth planets oppose!",
                "Earth blocks network packets!"));

        let jup_pos = positions.iter().find(|p| p.planet == Planet::Jupiter).unwrap();
        let _ = writeln!(report, "💾 Memory-Heavy Tasks (Jupiter in {}): {}",
            jup_pos.sign.name(),
            task_status(Planet::Jupiter, Element::Water, Element::Fire, fire_water_clash,
                "Databases and caches optimized!",
                "Water fills memory but Fire planets oppose!",
                "Fire evaporates memory pools!"));

        let sat_pos = positions.iter().find(|p| p.planet == Planet::Saturn).unwrap();
        let _ = writeln!(report, "⚙️  System Tasks (Saturn in {}): {}",
            sat_pos.sign.name(),
            task_status(Planet::Saturn, Element::Earth, Element::Air, earth_air_clash,
                "System operations rock solid!",
                "Earth stabilizes systems but Air planets oppose!",
                "Air disrupts system stability!"));

        // Element summary
        report.push_str("\n📊 Elemental Balance:\n");

        let _ = writeln!(report, "   Fire (CPU): {fire_count} planets | Earth (Stability): {earth_count} planets");
        let _ = writeln!(report, "   Air (Network): {air_count} planets | Water (Memory): {water_count} planets");

        // Check for elemental conflicts
        report.push_str("\n⚔️  Cosmic Tensions:\n");
        let mut has_tensions = false;

        if fire_count >= 2 && water_count >= 2 {
            let _ = writeln!(report, "   🔥💧 Fire vs Water CLASH! {fire_count} Fire planets battle {water_count} Water planets!");
            report.push_str("      CPU tasks and Memory tasks are in cosmic opposition!\n");
            has_tensions = true;
        }

        if earth_count >= 2 && air_count >= 2 {
            let _ = writeln!(report, "   🪨💨 Earth vs Air CLASH! {earth_count} Earth planets battle {air_count} Air planets!");
            report.push_str("      System tasks and Network tasks are in cosmic opposition!\n");
            has_tensions = true;
        }

        if !has_tensions {
            report.push_str("   ✌️  The elements are at peace (for now).\n");
        }

        // ── Planetary Aspects ──────────────────────────────────────────────────
        if enable_aspects {
            let aspects = calculate_aspects(&positions);
            report.push_str("\n🔭 Planetary Aspects:\n");
            if aspects.is_empty() {
                report.push_str("   No major aspects active — planets orbit in splendid isolation.\n");
            } else {
                for asp in &aspects {
                    let _ = writeln!(report,
                        "   {} {} {} {} (orb: {:.1}°, strength: {:.0}%)",
                        asp.planet_a.name(),
                        asp.aspect_type.symbol(),
                        asp.planet_b.name(),
                        asp.aspect_type.name(),
                        asp.orb,
                        asp.strength * 100.0,
                    );
                }
            }

            // Show aspects for each ruling planet
            report.push_str("\n   Aspect influences on task types:\n");
            for &tt in &[TaskType::CpuIntensive, TaskType::Network, TaskType::MemoryHeavy, TaskType::System] {
                let planet = tt.ruling_planet();
                let modifier = combined_aspect_modifier(&aspects, planet);
                let desc = describe_aspects(&aspects, planet);
                let _ = writeln!(report, "   {} {}: {:.0}% — {}",
                    tt.name(), planet.name(), modifier * 100.0, desc);
            }
        }

        // ── Moon Phase Tidal Effects ────────────────────────────────────────────
        if let Some(moon_pos) = positions.iter().find(|p| p.planet == Planet::Moon) {
            if let Some(phase) = moon_pos.moon_phase {
                report.push_str("\n🌊 Moon Phase Tidal Effects:\n");
                let interactive_mod = Self::moon_phase_modifier(phase);
                let memory_mod = Self::moon_phase_tidal_modifier(phase);
                let network_mod = Self::moon_phase_flow_modifier(phase);
                let _ = writeln!(report, "   Current phase: {} — {:+.0}% Interactive | {:+.0}% Memory | {:+.0}% Network",
                    phase.name(),
                    (interactive_mod - 1.0) * 100.0,
                    (memory_mod - 1.0) * 100.0,
                    (network_mod - 1.0) * 100.0,
                );
            }
        }

        // ── CPU Affinity Legend ────────────────────────────────────────────────
        if enable_cpu_affinity {
            report.push_str("\n🖥️  CPU Elemental Affinity (cycles every 4 CPUs):\n");
            for (idx, (elem, sign)) in [
                (Element::Fire,  element_sign(Element::Fire)),
                (Element::Earth, element_sign(Element::Earth)),
                (Element::Air,   element_sign(Element::Air)),
                (Element::Water, element_sign(Element::Water)),
            ].iter().enumerate() {
                let _ = writeln!(report, "   CPU {},{},{}… → {} ({}) — best for {} tasks",
                    idx, idx + 4, idx + 8,
                    elem.name(), sign,
                    match elem {
                        Element::Fire  => "CPU-Intensive",
                        Element::Earth => "System",
                        Element::Air   => "Network",
                        Element::Water => "Memory-Heavy",
                    }
                );
            }
        }

        report
    }
}

impl Default for AstrologicalScheduler {
    fn default() -> Self {
        Self::new(300) // Default to 5 minutes (300 seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = AstrologicalScheduler::new(300);
        assert_eq!(scheduler.cache_duration_secs, 300);
    }

    #[test]
    fn test_critical_task_priority() {
        let mut scheduler = AstrologicalScheduler::new(300);
        let now = Utc::now();

        let decision = scheduler.schedule_task("init", 1, now);

        assert_eq!(decision.priority, 1000);
        assert!(decision.reasoning.contains("CRITICAL"));
    }

    #[test]
    fn test_task_scheduling() {
        let mut scheduler = AstrologicalScheduler::new(300);
        let now = Utc::now();

        // Test various task types
        let firefox_decision = scheduler.schedule_task("firefox", 1234, now);
        assert!(firefox_decision.priority > 0);
        assert!(!firefox_decision.reasoning.is_empty());

        let rustc_decision = scheduler.schedule_task("rustc", 5678, now);
        assert!(rustc_decision.priority > 0);

        let systemd_decision = scheduler.schedule_task("systemd", 100, now);
        assert!(systemd_decision.priority > 0);
    }

    #[test]
    fn test_planetary_caching() {
        let mut scheduler = AstrologicalScheduler::new(300);
        let now = Utc::now();

        // First call should populate cache
        scheduler.schedule_task("bash", 1000, now);
        assert!(scheduler.planetary_cache.is_some());

        let cached_time = scheduler.planetary_cache.as_ref().unwrap().0;

        // Second call within cache window should reuse cache
        scheduler.schedule_task("vim", 1001, now);
        let still_cached_time = scheduler.planetary_cache.as_ref().unwrap().0;

        assert_eq!(cached_time, still_cached_time);
    }


    #[test]
    fn test_cosmic_weather_report() {
        let mut scheduler = AstrologicalScheduler::new(300);
        let now = Utc::now();

        let report = scheduler.get_cosmic_weather(now);

        assert!(report.contains("COSMIC WEATHER"));
        assert!(report.contains("Sun"));
        assert!(report.contains("Mercury"));
        assert!(report.contains("ASTROLOGICAL GUIDANCE"));
    }

    #[test]
    fn test_element_boost() {
        let now = Utc::now();
        let positions = calculate_planetary_positions(now);

        // Test that boosts are calculated
        let cpu_boost = AstrologicalScheduler::calculate_element_boost(&positions, TaskType::CpuIntensive);
        let net_boost = AstrologicalScheduler::calculate_element_boost(&positions, TaskType::Network);

        assert!(cpu_boost > 0.0);
        assert!(net_boost > 0.0);
    }

    #[test]
    fn test_planetary_influence() {
        let now = Utc::now();
        let positions = calculate_planetary_positions(now);

        for pos in positions {
            let influence = AstrologicalScheduler::calculate_planetary_influence(&pos);

            if pos.retrograde {
                // Retrograde planets have negative influence
                assert_eq!(influence, -1.0, "{} is retrograde and should have -1.0 influence", pos.planet.name());
            } else {
                // Direct planets have positive influence based on element
                assert!(influence > 0.0, "{} is direct and should have positive influence", pos.planet.name());
                assert!(influence >= 1.0 && influence <= 1.3, "{} influence should be between 1.0 and 1.3", pos.planet.name());
            }
        }
    }
}
