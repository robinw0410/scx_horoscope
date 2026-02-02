# Astrological Scheduling Formulas

## Zodiac Systems

By default, this scheduler uses the **traditional 12-sign tropical zodiac** with equal 30° divisions starting from the vernal equinox (0° Aries).

### Optional: 13-Sign Zodiac with Ophiuchus (`--ophiuchus` flag)

When the `--ophiuchus` flag is enabled, the scheduler uses the astronomically accurate 13-sign zodiac with IAU constellation boundaries, including **Ophiuchus** (the serpent-bearer) between Scorpio and Sagittarius.

| Sign | Ecliptic Longitude | Approximate Duration |
|------|-------------------|---------------------|
| Pisces | 351.5° - 29.0° | ~37° |
| Aries | 29.0° - 53.5° | ~24.5° |
| Taurus | 53.5° - 90.5° | ~37° |
| Gemini | 90.5° - 118.0° | ~27.5° |
| Cancer | 118.0° - 138.0° | ~20° |
| Leo | 138.0° - 174.0° | ~36° |
| Virgo | 174.0° - 218.0° | ~44° |
| Libra | 218.0° - 241.0° | ~23° |
| Scorpio | 241.0° - 248.0° | ~7° |
| **Ophiuchus** | **248.0° - 266.0°** | **~18°** |
| Sagittarius | 266.0° - 299.5° | ~33.5° |
| Capricorn | 299.5° - 327.5° | ~28° |
| Aquarius | 327.5° - 351.5° | ~24° |

Note: In 13-sign mode, signs have unequal lengths. Scorpio is particularly narrow (~7°), while Virgo is the largest (~44°). Many traditional Scorpios will find themselves in Libra or Ophiuchus!

## Priority Calculation

```
final_priority = base_priority × planetary_influence × element_boost
```

### Base Priorities
```
Critical: 1000  |  System: 200  |  Interactive: 150
Desktop: 120    |  CPU/Network: 100  |  Memory: 80
```

### Planetary Rulerships
- **Mars** → CPU-Intensive
- **Mercury** → Network, Interactive
- **Jupiter** → Memory-Heavy
- **Saturn** → System
- **Venus** → Desktop/UI
- **Sun** → Critical tasks

## Planetary Influence

**Retrograde**: `-1.0` (applies 50% time slice penalty)

**Direct** (by element of zodiac sign):
- Fire (Aries, Leo, Sagittarius, Ophiuchus): `1.3`
- Air (Gemini, Libra, Aquarius): `1.2`
- Earth (Taurus, Virgo, Capricorn): `1.1`
- Water (Cancer, Scorpio, Pisces): `1.0`

## Element Boost/Debuff

**BOOSTED (1.3-1.5x)**
- Fire × CPU: 1.5
- Air × Network: 1.5
- Earth × System: 1.4
- Water × Memory: 1.3

**DEBUFFED (0.6-0.7x)** - Opposing elements
- Water × CPU: 0.6 (dampens fire)
- Earth × Network: 0.6 (blocks air)
- Air × System: 0.7 (disrupts earth)
- Fire × Memory: 0.7 (evaporates water)

**Neutral**: 1.0 (all other combinations)

## Retrograde Detection

```rust
delta = longitude_tomorrow - longitude_today

retrograde = if delta > 180.0: true      // crossed 360° backward
             else if delta < -180.0: false  // crossed 360° forward
             else: delta < 0.0              // normal backward motion
```

Sun and Moon never retrograde.

## Example

**rustc (CPU task), Mars in Scorpio (Water), direct:**
```
100 × 1.0 × 0.6 = 60  → DEBUFFED
```

**rustc (CPU task), Mars in Aries (Fire), direct:**
```
100 × 1.3 × 1.5 = 195  → BOOSTED
```

Positions cached 5min. Calculated via `astro` crate (real ephemeris data).
