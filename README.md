# Nova

![Rust](https://img.shields.io/badge/language-Rust-f74c00) ![License](https://img.shields.io/badge/license-Unlicense-green)

Terminal panel for amateur astronomers. Weather forecast, ephemeris, astronomical events — decides whether it's worth taking the telescope out tonight.

Rust feature port of [astropanel](https://github.com/isene/astropanel), built on [crust](https://github.com/isene/crust). Shares the ephemeris engine with [tock](https://github.com/isene/tock).

## Install

```bash
# Build from source
git clone https://github.com/isene/nova
cd nova
cargo build --release

# Or download prebuilt binary from releases (Linux/macOS, x86_64/aarch64)
```

## Usage

```bash
nova
```

First run creates `~/.nova/config.yml` with Oslo defaults. Edit it to set your location, or press `l` inside the app.

## Keys

| Key | Action |
|---|---|
| `j` / `DOWN` | Next day |
| `k` / `UP` | Previous day |
| `0` / `HOME` | Jump to today |
| `r` | Reload weather (from cache if < 3h old) |
| `R` | Force re-fetch weather |
| `l` | Set location (`Name lat,lon`) |
| `c` | Cloud limit % |
| `h` | Humidity limit % |
| `t` | Temperature lower limit °C |
| `w` | Wind limit m/s |
| `W` | Save config |
| `?` | Help |
| `q` / `ESC` | Quit |

## Configuration

`~/.nova/config.yml` (YAML):

```yaml
location: Oslo
lat: 59.91
lon: 10.75
tz: 1.0
cloud_limit: 40
humidity_limit: 80.0
temp_limit: -10.0
wind_limit: 8.0
show_planets: true
show_events: true
```

Weather cache lives at `~/.nova/weather_cache.json` (TTL 3 hours).

## Condition rules

Following astropanel's scoring. Each day gets "negative points":

- 2 points if cloud cover exceeds cloud_limit
- +1 point if cloud cover > (100 - cloud_limit)/2
- +1 point if cloud cover > 90%
- +1 point if humidity > humidity_limit
- +1 point if temperature < temp_limit
- +1 point if temperature < temp_limit - 7°C
- +1 point if wind > wind_limit
- +1 point if wind > 2 × wind_limit

**0-1 = GOOD (green), 2-3 = FAIR (yellow), 4+ = BAD (red).**

## Data sources

- **Weather**: [api.met.no](https://api.met.no/) (Norwegian Meteorological Institute)
- **Ephemeris**: IAU 2006 obliquity standard, ported from [ruby-ephemeris](https://github.com/isene/ephemeris)

## Part of the Rust Terminal Suite (Fe2O3)

- [rush](https://github.com/isene/rush) — shell
- [pointer](https://github.com/isene/pointer) — file manager
- [kastrup](https://github.com/isene/kastrup) — messaging hub
- [scroll](https://github.com/isene/scroll) — web browser
- [tock](https://github.com/isene/tock) — calendar
- **nova** — astronomy panel

## License

Unlicense (public domain).
