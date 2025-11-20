# scx_horoscope - Astrological CPU Scheduler

> "Why let mere mortals decide CPU priorities when the cosmos can guide us?"

A fully functional sched_ext scheduler that makes real CPU scheduling decisions based on real-time planetary positions, zodiac signs, and astrological principles. This actually loads into the Linux kernel and schedules your system tasks. Because if the universe can influence our lives, why not our CPU scheduling too?

![Demo](demo.gif)

## Features

- **Real Planetary Calculations**: Uses the `astro` crate for accurate geocentric planetary positions
- **Zodiac-Based Task Classification**: Tasks are classified by their astrological affinities
- **Retrograde Detection**: Real retrograde motion detection by comparing day-to-day positions - negative influences trigger 50% time slice penalties
- **Lunar Phase Scheduling**: Moon phases affect Interactive tasks (shells, editors) with Full Moon giving 1.4x boost
- **Element Boosts & Debuffs**: Fire signs boost CPU tasks (1.5x), Water signs debuff them (0.6x) - elemental oppositions create cosmic chaos
- **Cosmic Weather Reports**: Get real-time astrological guidance for your system with moon phase tracking
- **Actually Works**: Loads into the Linux kernel via sched_ext and schedules real system processes
- **Real BPF Integration**: Uses scx_rustland_core framework for kernel-userspace communication
- **Dynamic Time Slicing**: Adjusts CPU time based on astrological priority (100-1000)

## Astrological Scheduling Rules

### Planetary Domains

Each planet rules specific types of system tasks:

- **☀️ Sun** (Life Force): Critical system processes (PID 1, init)
- **🌙 Moon** (Emotions): Interactive tasks (shells, editors, terminals)
- **💬 Mercury** (Communication): Network and I/O tasks
- **💖 Venus** (Harmony): Desktop and UI processes
- **⚔️ Mars** (Energy): CPU-intensive tasks (compilers, video encoding)
- **🎯 Jupiter** (Expansion): Memory-heavy applications (databases, browsers)
- **⚙️ Saturn** (Structure): System daemons and kernel threads

### Element Effects

Zodiac sign elements create elemental affinities and oppositions:

**Boosted Combinations** (tasks thrive under compatible elements):
- **🔥 Fire** (Aries, Leo, Sagittarius) × CPU tasks: 1.5x boost
- **🌬️ Air** (Gemini, Libra, Aquarius) × Network tasks: 1.5x boost
- **🌍 Earth** (Taurus, Virgo, Capricorn) × System tasks: 1.4x boost
- **💧 Water** (Cancer, Scorpio, Pisces) × Memory tasks: 1.3x boost

**Debuffed Combinations** (elemental oppositions):
- **💧 Water** × CPU tasks: 0.6x (water dampens fire)
- **🌍 Earth** × Network tasks: 0.6x (earth blocks air)
- **🌬️ Air** × System tasks: 0.7x (air disrupts earth's structure)
- **🔥 Fire** × Memory tasks: 0.7x (fire evaporates water)

All other combinations are neutral (1.0x).

### Retrograde Effects

When a planet is in retrograde motion, tasks under its domain suffer a **50% time slice penalty**. Retrograde is detected by comparing daily planetary positions - when a planet moves backward through the zodiac (negative delta in ecliptic longitude), it's retrograde.

Key retrograde effects:
- **Mercury Retrograde**: Network and interactive tasks suffer
- **Mars Retrograde**: CPU tasks crawl like molasses
- **Venus Retrograde**: UI becomes disharmonious
- **Note**: Sun and Moon never go retrograde

### Priority Formula

```
final_priority = base_priority × planetary_influence × element_boost
time_slice = min_slice + (base_slice - min_slice) × (priority / 1000)
if retrograde: time_slice × 0.5
```

**Base Priorities:**
- Critical (PID 1): 1000
- System tasks: 200
- Interactive tasks: 150
- Desktop/UI: 120
- CPU/Network: 100
- Memory: 80

**Planetary Influence (when planet is direct):**
- Fire signs: 1.3x
- Air signs: 1.2x
- Earth signs: 1.1x
- Water signs: 1.0x
- Retrograde: -1.0 (triggers time slice penalty)

## Installation

```bash
cargo build --release
```

## Usage

**Requires root privileges** to load into the kernel as a sched_ext scheduler.

### Running the Scheduler

```bash
# Build
cargo build --release

# Run with cosmic weather report and verbose output
sudo target/release/scx_horoscope --cosmic-weather -v

# Run with debug decisions to see individual task scheduling
sudo target/release/scx_horoscope --debug-decisions

# Stop: Press Ctrl+C for graceful shutdown
```

### Command-Line Options

**Astrological Options:**
- `-w, --cosmic-weather` - Display planetary positions and astrological guidance on startup
- `-d, --debug-decisions` - Watch the cosmos make scheduling decisions in real-time
- `--no-retrograde` - Boring mode (disables retrograde chaos)
- `-u, --update-interval <SECS>` - Update planetary positions every N seconds (default: 60)

**Performance Tuning:**
- `-s, --slice-us <MICROSECONDS>` - Base time slice duration (default: 5000)
- `--slice-us-min <MICROSECONDS>` - Minimum time slice (default: 500)
- `-v, --verbose` - Display detailed statistics

Run `--help` to see all options.

## Disclaimer

This scheduler is **100% for educational and entertainment purposes**. While the astronomical calculations are real and the scheduler actually works (it really does load into the kernel and schedule tasks!), using astrology to schedule CPU tasks is:

- Scientifically dubious
- Cosmically hilarious
- Fully functional with real retrograde detection and lunar phase scheduling
- Not recommended for production systems (but it boots and runs stably)
- Perfect for conference talks, hackathons, and proving that anything is possible

## License

GPL-2.0-only (as required by sched_ext)

## Contributing

Contributions welcome! The core scheduler is working, but there's always room for more cosmic chaos:

- Add more planetary aspects (conjunctions, oppositions, trines)
- Add birth chart generation for processes (based on creation time)
- Horoscope predictions for task completion times
- Per-CPU affinity based on astrological compatibility
- Extend moon phase effects to other task types (I/O, memory operations)

## Acknowledgments

- **Norm** for the hilarious and brilliant idea to schedule by the stars
- **sched_ext** team for the amazing BPF scheduler framework
- The cosmos, for providing endless entertainment

---

*"In space, no one can hear you schedule."*

Comments from people:

- "You've made it work, but at what cost!"

