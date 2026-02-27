use super::planets::{Planet, PlanetaryPosition, Element};
use super::tasks::TaskType;
use chrono::{DateTime, Utc, Datelike, Timelike};

/// Estimated time range for task completion, with cosmic justification.
#[derive(Debug, Clone)]
pub struct CompletionTimeRange {
    pub min_seconds: u64,
    pub max_seconds: u64,
    /// 0.0 = "I asked a Magic 8-Ball", 1.0 = "Newtonian certainty"
    pub confidence: f64,
    pub cosmic_reason: String,
}

/// A full horoscope prediction for a given task type.
#[derive(Debug, Clone)]
pub struct HoroscopePrediction {
    pub task_type: TaskType,
    pub ruling_planet: Planet,
    pub completion_estimate: CompletionTimeRange,
    pub daily_forecast: String,
    pub lucky_number: u8,
    /// 0-23: the best hour of day to run this task type, per planetary wisdom
    pub power_hour: u8,
}

fn task_type_ordinal(t: TaskType) -> u64 {
    match t {
        TaskType::Network      => 0,
        TaskType::CpuIntensive => 1,
        TaskType::Desktop      => 2,
        TaskType::MemoryHeavy  => 3,
        TaskType::System       => 4,
        TaskType::Interactive  => 5,
        TaskType::Critical     => 6,
    }
}

impl HoroscopePrediction {
    pub fn generate(
        positions: &[PlanetaryPosition],
        task_type: TaskType,
        now: &DateTime<Utc>,
    ) -> Self {
        let ruling_planet = task_type.ruling_planet();

        // Find ruling planet's position (fall back to a synthetic one if absent)
        let ruling_pos = positions
            .iter()
            .find(|p| p.planet == ruling_planet)
            .cloned()
            .unwrap_or_else(|| PlanetaryPosition {
                planet: ruling_planet,
                longitude: 0.0,
                sign: super::planets::ZodiacSign::Aries,
                retrograde: false,
                moon_phase: None,
            });

        let element = ruling_pos.sign.element();
        let sign_name = ruling_pos.sign.name();
        let planet_name = ruling_planet.name();

        // ── Base time ranges (min_s, max_s) per task type ──────────────────
        let (base_min, base_max): (u64, u64) = match task_type {
            TaskType::CpuIntensive => (120,  3600),
            TaskType::Network      => (5,    300),
            TaskType::MemoryHeavy  => (60,   1800),
            TaskType::System       => (1,    60),
            TaskType::Interactive  => (1,    30),
            TaskType::Desktop      => (1,    10),
            TaskType::Critical     => (1,    1),
        };

        // ── Retrograde multiplier ──────────────────────────────────────────
        let retro_multiplier: f64 = if ruling_pos.retrograde { 3.0 } else { 1.0 };

        // ── Element speed modifier (Fire=fast, Earth=slow, Air/Water=medium) ─
        let element_multiplier: f64 = match element {
            Element::Fire  => 0.75,
            Element::Earth => 1.50,
            Element::Air   => 1.00,
            Element::Water => 1.25,
        };

        let combined = retro_multiplier * element_multiplier;
        let adj_min = ((base_min as f64) * combined).round() as u64;
        let adj_max = ((base_max as f64) * combined).round() as u64;
        // Clamp so min is always at least 1
        let final_min = adj_min.max(1);
        let final_max = adj_max.max(final_min);

        // ── Confidence ────────────────────────────────────────────────────
        // Retrograde tanks confidence; earthy stability helps it.
        let confidence: f64 = {
            let base = if ruling_pos.retrograde { 0.45 } else { 0.80 };
            let element_bonus: f64 = match element {
                Element::Earth => 0.15,
                Element::Fire  => 0.05,
                Element::Air   => 0.00,
                Element::Water => -0.05,
            };
            (base + element_bonus).clamp(0.05, 1.0)
        };

        // ── Cosmic reason string ──────────────────────────────────────────
        let motion = if ruling_pos.retrograde { "retrograde" } else { "direct" };
        let element_quip: &str = match element {
            Element::Fire  => match task_type {
                TaskType::CpuIntensive => "CPU burns bright!",
                TaskType::Network      => "packets ignite like solar flares!",
                TaskType::Desktop      => "pixels dance with fiery grace!",
                TaskType::MemoryHeavy  => "RAM blazes with celestial fuel!",
                TaskType::System       => "kernel ignites with volcanic precision!",
                TaskType::Interactive  => "your shell crackles with warmth!",
                TaskType::Critical     => "the cosmos holds its breath!",
            },
            Element::Earth => match task_type {
                TaskType::CpuIntensive => "compilation plods but endures!",
                TaskType::Network      => "packets travel like tectonic plates!",
                TaskType::Desktop      => "pixels settle, unmoving as mountains!",
                TaskType::MemoryHeavy  => "memory grows slow and deep as bedrock!",
                TaskType::System       => "the kernel stands firm as granite!",
                TaskType::Interactive  => "your keystrokes echo through stone!",
                TaskType::Critical     => "the foundation holds all things!",
            },
            Element::Air   => match task_type {
                TaskType::CpuIntensive => "threads drift on cosmic winds!",
                TaskType::Network      => "packets flow like starlight!",
                TaskType::Desktop      => "windows flutter like butterfly wings!",
                TaskType::MemoryHeavy  => "heap allocations float on gentle breeze!",
                TaskType::System       => "daemons whisper through the ether!",
                TaskType::Interactive  => "your commands ride the celestial zephyr!",
                TaskType::Critical     => "the vital spark breathes free!",
            },
            Element::Water => match task_type {
                TaskType::CpuIntensive => "cycles flow like a mountain stream!",
                TaskType::Network      => "packets ripple through digital tides!",
                TaskType::Desktop      => "frames cascade like gentle waterfalls!",
                TaskType::MemoryHeavy  => "data pools in deep lunar lakes!",
                TaskType::System       => "the kernel ebbs and flows!",
                TaskType::Interactive  => "your session swirls in the cosmic ocean!",
                TaskType::Critical     => "the lifeblood of the machine churns!",
            },
        };

        let cosmic_reason = format!(
            "{} {} in {} ({}) — {}",
            planet_name, motion, sign_name, element.name(), element_quip
        );

        // ── Daily forecast ────────────────────────────────────────────────
        let daily_forecast = generate_daily_forecast(
            ruling_planet,
            &ruling_pos,
            task_type,
            now,
        );

        // ── Lucky number ──────────────────────────────────────────────────
        let lucky_number = ((ruling_pos.longitude as u64 + task_type_ordinal(task_type)) % 99 + 1) as u8;

        // ── Power hour ────────────────────────────────────────────────────
        let power_hour: u8 = match ruling_planet {
            Planet::Sun     => 12,
            Planet::Moon    => 0,
            Planet::Mercury => 9,
            Planet::Venus   => 18,
            Planet::Mars    => 6,
            Planet::Jupiter => 15,
            Planet::Saturn  => 21,
        };

        HoroscopePrediction {
            task_type,
            ruling_planet,
            completion_estimate: CompletionTimeRange {
                min_seconds: final_min,
                max_seconds: final_max,
                confidence,
                cosmic_reason,
            },
            daily_forecast,
            lucky_number,
            power_hour,
        }
    }

}

fn generate_daily_forecast(
    planet: Planet,
    pos: &PlanetaryPosition,
    _task_type: TaskType,
    now: &DateTime<Utc>,
) -> String {
    let sign = pos.sign.name();
    let element = pos.sign.element();
    let retro = pos.retrograde;
    // Use hour to add slight time-of-day flavor
    let hour = now.hour();

    // Moon phase flavoring for Moon-ruled tasks
    let moon_phase_note = if planet == Planet::Moon {
        if let Some(phase) = pos.moon_phase {
            format!(" The {} Moon amplifies your intentions.", phase.name())
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // 6+ variations per planet based on sign element + retrograde
    let base_forecast = match planet {
        Planet::Mars => {
            if retro {
                match element {
                    Element::Fire => format!(
                        "Mars retreats through {}, its war-drums muffled by cosmic static. \
                         Your CPU cores are present, but emotionally unavailable. \
                         Recompile with lower expectations — and maybe a snack.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Mars trudges backward through {}, dragging its compile flags through \
                         the mud. Progress is measured in geological epochs. \
                         Consider a nap. Or three.",
                        sign
                    ),
                    Element::Air => format!(
                        "Mars spirals retrograde through airy {}, scattering your \
                         build artifacts like autumn leaves. The linker weeps. \
                         Your patience will be tested by the cosmos and the cache.",
                        sign
                    ),
                    Element::Water => format!(
                        "Mars drifts backward through watery {}, your compilation \
                         submerged in cosmic molasses. Threads stall. Cores dream. \
                         Breathe deep — even Olympus had compile errors.",
                        sign
                    ),
                }
            } else {
                match element {
                    Element::Fire => format!(
                        "Mars blazes through {} with reckless abandon! Your compilation \
                         burns with cosmic fire. Expect the unexpected — the stars promise \
                         chaos, but glorious chaos!",
                        sign
                    ),
                    Element::Earth => format!(
                        "Mars marches steadily through {}, your CPU cores disciplined \
                         as Roman legions. Compilation proceeds with earthy determination. \
                         Slow? Yes. But it will finish. Eventually.",
                        sign
                    ),
                    Element::Air => format!(
                        "Mars dances through {}, threads pirouetting on celestial breezes. \
                         Your code compiles swiftly, borne aloft by cosmic tailwinds. \
                         May your linker be ever in your favor.",
                        sign
                    ),
                    Element::Water => format!(
                        "Mars flows through {}, your build process surging like a river \
                         in flood. Powerful, fluid, slightly unpredictable. \
                         Keep your Makefile dry.",
                        sign
                    ),
                }
            }
        }

        Planet::Mercury => {
            if retro {
                match element {
                    Element::Fire => format!(
                        "Mercury retrograde scorches through {}! Packets combust spontaneously. \
                         Your SSH sessions drop not from server error, but from cosmic disapproval. \
                         Back up your configs. The stars recommend it urgently.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Mercury crawls backward through {}, dragging your TCP handshakes \
                         through sedimentary time. Every ping a geological event. \
                         Consider carrier pigeons as a fallback protocol.",
                        sign
                    ),
                    Element::Air => format!(
                        "Mercury double-backs through {}, scattering packets to the four winds. \
                         DNS resolves to yesterday. TLS certificates have opinions. \
                         The universe apologizes for the inconvenience.",
                        sign
                    ),
                    Element::Water => format!(
                        "Mercury swims upstream through {}, your network traffic eddying \
                         in cosmic backwash. Latency spikes. Connections time out philosophically. \
                         This too shall pass — but slowly.",
                        sign
                    ),
                }
            } else {
                match element {
                    Element::Fire => format!(
                        "Mercury races through fiery {}! Packets ignite with purpose, \
                         your network blazing trails through the ether. \
                         Download speeds ascend. The gods of bandwidth smile.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Mercury grounds itself in {}, your network traffic steady and reliable \
                         as tectonic plates — if slightly slower. \
                         Reliable. Predictable. Boringly excellent.",
                        sign
                    ),
                    Element::Air => format!(
                        "Mercury soars through {}, packets flowing like starlight through the void! \
                         Your connections snap into place with divine efficiency. \
                         The cosmic routers are on your side today.",
                        sign
                    ),
                    Element::Water => format!(
                        "Mercury drifts through {}, your data streams rippling with gentle purpose. \
                         Network traffic flows in tranquil currents. \
                         Latency is low. The ocean of bits is calm.",
                        sign
                    ),
                }
            }
        }

        Planet::Venus => {
            if retro {
                match element {
                    Element::Fire => format!(
                        "Venus retreats through fiery {}! Your desktop flickers with \
                         unrequited rendering. Windows refuse to redraw. Icons sulk. \
                         Your compositor needs a hug it won't receive.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Venus trudges backward through {}, your UI elements stubbornly \
                         refusing to refresh. Pixel art from a bygone era haunts your screen. \
                         Embrace the aesthetic. It is the aesthetic now.",
                        sign
                    ),
                    Element::Air => format!(
                        "Venus floats backward through {}, scattering your window layouts \
                         to the cosmic breeze. Tiling managers weep. Floating windows \
                         float where they will. Resistance is futile and also not calming.",
                        sign
                    ),
                    Element::Water => format!(
                        "Venus dissolves backward through {}, your screen smearing \
                         like watercolors in rain. Frame rate dips. Vsync becomes \
                         an aspiration rather than a guarantee.",
                        sign
                    ),
                }
            } else {
                match element {
                    Element::Fire => format!(
                        "Venus dances through {}, your desktop ablaze with gorgeous frames! \
                         Animations are buttery. Icons glow with inner radiance. \
                         Your compositor is having the best day of its life.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Venus settles into {}, your UI solid and dependable as good furniture. \
                         No flashy transitions — just honest pixels, rendered with dignity. \
                         Sturdy. Reliable. Beautifully understated.",
                        sign
                    ),
                    Element::Air => format!(
                        "Venus glides through {}, your desktop elegant as a Viennese waltz. \
                         Transitions flow. Fonts kern perfectly. \
                         Even your terminal emulator looks a little glamorous today.",
                        sign
                    ),
                    Element::Water => format!(
                        "Venus flows through {}, your interface rippling with liquid grace. \
                         Smooth gradients. Silky scrolling. \
                         Your GTK theme has never looked so spiritually nourishing.",
                        sign
                    ),
                }
            }
        }

        Planet::Jupiter => {
            if retro {
                match element {
                    Element::Fire => format!(
                        "Jupiter retreats through {}, your heap contracting with retrograde \
                         indignation. malloc returns NULL as a lifestyle choice. \
                         The JVM considers early retirement. Swap space expands to fill the void.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Jupiter lumbers backward through {}, memory allocations grinding \
                         to a geological pace. Your database ponders each query with \
                         the philosophical weight of ages. Patience is a virtue; RAM is a commodity.",
                        sign
                    ),
                    Element::Air => format!(
                        "Jupiter spirals backward through {}, heap objects drifting loose \
                         from their pointers. Memory leaks with cosmic abandon. \
                         The garbage collector runs, but it too is retrograde.",
                        sign
                    ),
                    Element::Water => format!(
                        "Jupiter drifts backward through {}, your memory pool swirling \
                         in retrograde eddies. Cache evictions are frequent and sorrowful. \
                         Even Redis is having feelings about it.",
                        sign
                    ),
                }
            } else {
                match element {
                    Element::Fire => format!(
                        "Jupiter expands through {}, your memory abundant as cosmic fire! \
                         The JVM allocates with divine extravagance. \
                         Heap size is merely a suggestion. The stars say: request more RAM.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Jupiter grounds through {}, your memory management solid and \
                         methodical. Pages are allocated with deliberate care. \
                         Your database grows slowly, like an oak — but magnificent.",
                        sign
                    ),
                    Element::Air => format!(
                        "Jupiter soars through {}, your application breathing freely in \
                         vast memory expanses! Objects instantiate with joyful abandon. \
                         The heap is open sky. Fly, little processes, fly.",
                        sign
                    ),
                    Element::Water => format!(
                        "Jupiter flows through {}, your memory pools deep and full. \
                         Allocations swim gracefully. The buffer blooms with plenty. \
                         Even valgrind is impressed, and valgrind is never impressed.",
                        sign
                    ),
                }
            }
        }

        Planet::Saturn => {
            if retro {
                match element {
                    Element::Fire => format!(
                        "Saturn retreats through {}, system daemons questioning their purpose. \
                         Cron jobs run when the mood strikes. systemd unit files develop \
                         strong opinions about dependency order. Structure unravels, cosmically.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Saturn grinds backward through {}, your system calls taking the \
                         scenic route through kernel space. Everything works. \
                         Just... not enthusiastically. The system endures.",
                        sign
                    ),
                    Element::Air => format!(
                        "Saturn drifts backward through {}, daemons whispering conflicting \
                         instructions to the scheduler. IRQ storms gather on the horizon. \
                         Consider rebooting. The stars would understand.",
                        sign
                    ),
                    Element::Water => format!(
                        "Saturn ebbs backward through {}, kernel threads pooling in \
                        existential eddies. The init system has doubts. \
                         journald logs feelings instead of events.",
                        sign
                    ),
                }
            } else {
                match element {
                    Element::Fire => format!(
                        "Saturn marches through {}, system services snapping to attention! \
                         Daemons start promptly. Cron jobs execute with military precision. \
                         The kernel is on time, on task, and mildly intimidating.",
                        sign
                    ),
                    Element::Earth => format!(
                        "Saturn settles through {}, your system stable as continental bedrock. \
                         Every syscall lands cleanly. Every daemon persists reliably. \
                         This is what peak kernel performance looks like.",
                        sign
                    ),
                    Element::Air => format!(
                        "Saturn glides through {}, system processes light and efficient. \
                         Interrupt latency is a pleasant fiction. \
                         The scheduler hums in tune with the celestial spheres.",
                        sign
                    ),
                    Element::Water => format!(
                        "Saturn flows through {}, kernel threads streaming in orderly \
                         procession. System calls return swiftly and without drama. \
                         The OS is in flow state. Appreciate it while it lasts.",
                        sign
                    ),
                }
            }
        }

        Planet::Moon => {
            let phase_str = pos.moon_phase.map(|p| p.name()).unwrap_or("present");
            if retro {
                // Moon doesn't retrograde, but handle gracefully
                format!(
                    "The {} Moon lingers in {}, your shell sessions contemplative and slow. \
                     Tab completion pauses to reflect on the nature of file paths. \
                     Terminal emulators emit soft sighs.{}",
                    phase_str, sign, moon_phase_note
                )
            } else {
                match element {
                    Element::Fire => format!(
                        "The {} Moon blazes through {}, your terminal crackles with \
                         interactive energy! Commands execute with passion. \
                         Even `ls` feels thrilling.{}",
                        phase_str, sign, moon_phase_note
                    ),
                    Element::Earth => format!(
                        "The {} Moon grounds through {}, your interactive sessions \
                         steady and methodical. Bash history is consulted wisely. \
                         Typos are minimal. The cosmos rewards deliberate typing.{}",
                        phase_str, sign, moon_phase_note
                    ),
                    Element::Air => format!(
                        "The {} Moon drifts through {}, your shell sessions light and \
                         responsive as a feather! Autocomplete flows like poetry. \
                         The terminal is your oyster.{}",
                        phase_str, sign, moon_phase_note
                    ),
                    Element::Water => format!(
                        "The {} Moon glimmers through {}, your interactive tasks \
                         flowing in gentle rhythms. Pipe chains cascade like waterfalls. \
                         Your awk one-liners are briefly beautiful.{}",
                        phase_str, sign, moon_phase_note
                    ),
                }
            }
        }

        Planet::Sun => {
            // Critical tasks ruled by the Sun
            let time_blessing = if hour >= 10 && hour <= 14 {
                "The Sun is near its zenith — critical processes bask in full solar power."
            } else if hour < 6 || hour >= 20 {
                "The Sun slumbers below the horizon, yet critical processes never sleep."
            } else {
                "The Sun blesses this hour with steady, life-giving radiance."
            };
            format!(
                "The Sun holds court in {}, and your most critical processes are \
                 under direct divine patronage. PID 1 does not negotiate with retrograde. \
                 {} All hail the init system.",
                sign, time_blessing
            )
        }
    };

    base_forecast
}

pub fn format_duration(min_s: u64, max_s: u64) -> String {
    // Choose unit based on the larger end of the range
    if max_s < 120 {
        // Show in seconds
        if min_s == max_s {
            format!("{} second{}", min_s, if min_s == 1 { "" } else { "s" })
        } else {
            format!("{}-{} seconds", min_s, max_s)
        }
    } else if max_s < 3600 {
        // Show in minutes
        let min_m = (min_s + 59) / 60; // ceil
        let max_m = max_s / 60;        // floor
        let min_m = min_m.max(1);
        let max_m = max_m.max(min_m);
        if min_m == max_m {
            format!("{} minute{}", min_m, if min_m == 1 { "" } else { "s" })
        } else {
            format!("{}-{} minutes", min_m, max_m)
        }
    } else {
        // Show in hours
        let min_h = (min_s + 3599) / 3600; // ceil
        let max_h = max_s / 3600;           // floor
        let min_h = min_h.max(1);
        let max_h = max_h.max(min_h);
        if min_h == max_h {
            format!("{} hour{}", min_h, if min_h == 1 { "" } else { "s" })
        } else {
            format!("{}-{} hours", min_h, max_h)
        }
    }
}

pub fn confidence_stars(confidence: f64) -> String {
    let filled = (confidence * 5.0).round() as usize;
    let filled = filled.min(5);
    let empty = 5 - filled;
    format!("{}{}", "★".repeat(filled), "☆".repeat(empty))
}

fn word_wrap(text: &str, width: usize, indent: &str) -> String {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(format!("{}{}", indent, current_line));
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(format!("{}{}", indent, current_line));
    }
    lines.join("\n")
}

// Cosmic wisdom quotes, rotated by day of month
const COSMIC_WISDOM: &[&str] = &[
    "\"The CPU is the mind, RAM the imagination, and disk the long memory of the machine. \
     Tend them as you would tend your soul.\"",
    "\"Mercury retrograde is not an excuse — it is an explanation. There is a difference, \
     and your coworkers will not appreciate the distinction.\"",
    "\"A process that completes under Mars is forged in fire. \
     A process that completes under Saturn merely completes. Both are worthy.\"",
    "\"The scheduler does not play favorites. The planets, however, absolutely do.\"",
    "\"Even the mightiest kernel panic is written in the stars. \
     This does not make it less catastrophic.\"",
    "\"Jupiter in retrograde is why your Electron app uses 8 GB of RAM. \
     Jupiter direct is why it only uses 6 GB. Plan accordingly.\"",
    "\"The Full Moon brings peak interactivity and also, inexplicably, \
     more Slack notifications. This is not a coincidence.\"",
    "\"Saturn rules structure, discipline, and the /etc directory. \
     Do not modify /etc during a Saturn retrograde. You have been warned.\"",
];

pub fn get_system_daily_horoscope(
    positions: &[PlanetaryPosition],
    now: &DateTime<Utc>,
) -> String {
    let mut out = String::new();

    // ── Date header ──────────────────────────────────────────────────────────
    out.push_str(&format!(
        "\
╔══════════════════════════════════════════════════════════════╗
║         \u{2728} SYSTEM DAILY HOROSCOPE — {} ✨         ║
╚══════════════════════════════════════════════════════════════╝

",
        now.format("%Y-%m-%d %H:%M UTC")
    ));

    // ── Planets of the day ───────────────────────────────────────────────────
    out.push_str("\u{1F315} PLANETS OF THE DAY\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");

    // Find the most "influential" planets: prefer direct planets, then retrograde
    let mut planet_lines: Vec<String> = Vec::new();
    for pos in positions {
        let retro_tag = if pos.retrograde { " ℞ RETROGRADE" } else { "" };
        let moon_tag = if let Some(phase) = pos.moon_phase {
            format!(" [{}]", phase.name())
        } else {
            String::new()
        };
        planet_lines.push(format!(
            "  {:8} in {:12} ({:5}){}{} — {}",
            pos.planet.name(),
            pos.sign.name(),
            pos.sign.element().name(),
            retro_tag,
            moon_tag,
            pos.planet.domain(),
        ));
    }
    // Show direct planets first, then retrograde
    planet_lines.sort_by_key(|l| if l.contains("RETROGRADE") { 1u8 } else { 0u8 });
    for line in &planet_lines {
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');

    // ── Per-task-type guidance ───────────────────────────────────────────────
    out.push_str("\u{1F4CB} TASK TYPE GUIDANCE\n");
    out.push_str("──────────────────────────────────────────────────────────────\n\n");

    let all_tasks = [
        TaskType::Critical,
        TaskType::CpuIntensive,
        TaskType::Network,
        TaskType::MemoryHeavy,
        TaskType::System,
        TaskType::Desktop,
        TaskType::Interactive,
    ];

    for &task in &all_tasks {
        let prediction = HoroscopePrediction::generate(positions, task, now);
        let est = &prediction.completion_estimate;
        let duration = format_duration(est.min_seconds, est.max_seconds);
        let stars = confidence_stars(est.confidence);

        out.push_str(&format!(
            "\u{1F538} {} (ruled by {})\n",
            task.name(),
            prediction.ruling_planet.name()
        ));
        out.push_str(&format!(
            "   Estimated: {}  |  Confidence: {}  |  Power hour: {:02}:00  |  Lucky #: {}\n",
            duration,
            stars,
            prediction.power_hour,
            prediction.lucky_number,
        ));
        // Wrap the forecast at 60 chars
        out.push_str(&word_wrap(&prediction.daily_forecast, 60, "   "));
        out.push('\n');
        out.push_str(&format!("   \u{1F30C} {}\n", est.cosmic_reason));
        out.push('\n');
    }

    // ── Cosmic wisdom quote (rotate by day of month) ─────────────────────────
    let day_idx = (now.day0() as usize) % COSMIC_WISDOM.len();
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str("\u{1F4AB} COSMIC WISDOM OF THE DAY:\n\n   ");
    out.push_str(COSMIC_WISDOM[day_idx]);
    out.push_str("\n\n");
    out.push_str("══════════════════════════════════════════════════════════════\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::planets::{ZodiacSign, MoonPhase, calculate_planetary_positions};
    use chrono::TimeZone;

    /// Build a minimal set of positions without calling the real astro crate.
    fn mock_positions() -> Vec<PlanetaryPosition> {
        vec![
            PlanetaryPosition {
                planet: Planet::Sun,
                longitude: 45.0,
                sign: ZodiacSign::Taurus,
                retrograde: false,
                moon_phase: None,
            },
            PlanetaryPosition {
                planet: Planet::Moon,
                longitude: 120.0,
                sign: ZodiacSign::Leo,
                retrograde: false,
                moon_phase: Some(MoonPhase::FirstQuarter),
            },
            PlanetaryPosition {
                planet: Planet::Mercury,
                longitude: 200.0,
                sign: ZodiacSign::Libra,
                retrograde: false,
                moon_phase: None,
            },
            PlanetaryPosition {
                planet: Planet::Venus,
                longitude: 300.0,
                sign: ZodiacSign::Aquarius,
                retrograde: false,
                moon_phase: None,
            },
            PlanetaryPosition {
                planet: Planet::Mars,
                longitude: 15.0,
                sign: ZodiacSign::Aries,
                retrograde: false,
                moon_phase: None,
            },
            PlanetaryPosition {
                planet: Planet::Jupiter,
                longitude: 90.0,
                sign: ZodiacSign::Cancer,
                retrograde: true,
                moon_phase: None,
            },
            PlanetaryPosition {
                planet: Planet::Saturn,
                longitude: 330.0,
                sign: ZodiacSign::Pisces,
                retrograde: false,
                moon_phase: None,
            },
        ]
    }

    fn test_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 2, 27, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_prediction_generation() {
        let positions = mock_positions();
        let now = test_now();

        let task_types = [
            TaskType::Network,
            TaskType::CpuIntensive,
            TaskType::Desktop,
            TaskType::MemoryHeavy,
            TaskType::System,
            TaskType::Interactive,
            TaskType::Critical,
        ];

        for &task in &task_types {
            let pred = HoroscopePrediction::generate(&positions, task, &now);

            // Basic sanity checks
            assert!(!pred.daily_forecast.is_empty(),
                "daily_forecast should not be empty for {:?}", task);
            assert!(!pred.completion_estimate.cosmic_reason.is_empty(),
                "cosmic_reason should not be empty for {:?}", task);
            assert!(pred.lucky_number >= 1 && pred.lucky_number <= 99,
                "lucky_number out of range for {:?}: {}", task, pred.lucky_number);
            assert!(pred.power_hour <= 23,
                "power_hour out of range for {:?}: {}", task, pred.power_hour);
            assert!(pred.completion_estimate.confidence >= 0.0
                && pred.completion_estimate.confidence <= 1.0,
                "confidence out of range for {:?}: {}", task, pred.completion_estimate.confidence);
            assert!(pred.completion_estimate.min_seconds >= 1,
                "min_seconds should be at least 1 for {:?}", task);
            assert!(pred.completion_estimate.max_seconds >= pred.completion_estimate.min_seconds,
                "max_seconds should be >= min_seconds for {:?}", task);
        }
    }

    #[test]
    fn test_format_duration() {
        // Small ranges → seconds
        assert_eq!(format_duration(1, 1), "1 second");
        assert_eq!(format_duration(5, 60), "5-60 seconds");
        assert_eq!(format_duration(30, 90), "30-90 seconds");
        assert!(format_duration(5, 60).contains("seconds"));

        // Medium ranges → minutes
        assert!(format_duration(60, 600).contains("minutes"));
        assert!(format_duration(120, 1800).contains("minutes"));
        let min_str = format_duration(120, 1800);
        assert!(min_str.contains("minute"), "Expected 'minute' in '{}'", min_str);

        // Large ranges → hours
        let hour_str = format_duration(3600, 14400);
        assert!(hour_str.contains("hour"), "Expected 'hour' in '{}'", hour_str);
        assert_eq!(format_duration(3600, 3600), "1 hour");
    }

    #[test]
    fn test_confidence_stars() {
        assert_eq!(confidence_stars(1.0), "★★★★★");
        assert_eq!(confidence_stars(0.0), "☆☆☆☆☆");

        // 0.8 → rounds to 4 filled
        let stars_80 = confidence_stars(0.8);
        assert_eq!(stars_80, "★★★★☆",
            "0.8 confidence should be 4 stars, got '{}'", stars_80);

        // 0.6 → rounds to 3 filled
        let stars_60 = confidence_stars(0.6);
        assert_eq!(stars_60, "★★★☆☆",
            "0.6 confidence should be 3 stars, got '{}'", stars_60);

        // Always 5 total characters
        for &c in &[0.0, 0.2, 0.4, 0.6, 0.8, 1.0] {
            let s = confidence_stars(c);
            let count: usize = s.chars().count();
            assert_eq!(count, 5, "confidence_stars({}) should have 5 chars, got {}", c, count);
        }
    }

    #[test]
    fn test_system_horoscope() {
        let positions = mock_positions();
        let now = test_now();

        let horoscope = get_system_daily_horoscope(&positions, &now);

        assert!(!horoscope.is_empty(), "System horoscope should not be empty");
        assert!(horoscope.contains("HOROSCOPE"),
            "System horoscope should contain 'HOROSCOPE'");
        assert!(horoscope.contains("PLANETS OF THE DAY"),
            "System horoscope should contain planets section");
        assert!(horoscope.contains("TASK TYPE GUIDANCE"),
            "System horoscope should contain task guidance section");
        assert!(horoscope.contains("COSMIC WISDOM"),
            "System horoscope should contain wisdom section");

        // Should mention all task type names
        for name in &["CPU-Intensive", "Network", "Memory-Heavy", "System",
                      "Desktop/UI", "Interactive", "Critical"] {
            assert!(horoscope.contains(name),
                "System horoscope should mention task type '{}'", name);
        }
    }

    /// Integration test: use real planetary positions from the astro crate
    /// (same date as existing planets.rs tests).
    #[test]
    fn test_real_positions_prediction() {
        let test_time = Utc.with_ymd_and_hms(2025, 11, 19, 22, 7, 46).unwrap();
        let positions = calculate_planetary_positions(test_time);

        let pred = HoroscopePrediction::generate(
            &positions,
            TaskType::CpuIntensive,
            &test_time,
        );

        // Mars should be the ruling planet for CpuIntensive
        assert_eq!(pred.ruling_planet, Planet::Mars);
        assert_eq!(pred.power_hour, 6);
        assert!(!pred.daily_forecast.is_empty());
        assert!(pred.daily_forecast.contains("Mars") || pred.completion_estimate.cosmic_reason.contains("Mars"));
    }
}
