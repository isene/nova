mod astronomy;
mod config;
mod date_util;
mod weather;

use crust::{Crust, Input, Pane};
use crust::style;
use config::Config;
use weather::DayForecast;

fn main() {
    config::ensure_dir();

    let mut cfg = Config::load();

    // First-run: prompt for location if still defaults.
    if cfg.location == "Oslo" && !std::path::Path::new(&config::config_path()).exists() {
        eprintln!("nova: no config found, using Oslo defaults. Edit {} to set your location.",
            config::config_path().display());
    }

    Crust::init();
    let mut app = App::new(cfg.clone());
    app.refresh_weather(false);
    app.render_all();

    loop {
        let Some(key) = Input::getchr(Some(60)) else {
            app.tick();
            continue;
        };
        match key.as_str() {
            "q" | "Q" | "ESC" => break,
            "?" => { app.show_help(); }
            "j" | "DOWN" => { app.move_day(1); app.render_all(); }
            "k" | "UP" => { app.move_day(-1); app.render_all(); }
            "HOME" | "0" => { app.go_today(); app.render_all(); }
            "r" => { app.refresh_weather(false); app.render_all(); }
            "R" => { app.refresh_weather(true); app.render_all(); }
            "l" => { app.prompt_location(); app.render_all(); cfg = app.cfg.clone(); let _ = cfg.save(); }
            "c" => { app.prompt_cloud(); app.render_all(); let _ = app.cfg.save(); }
            "h" => { app.prompt_humidity(); app.render_all(); let _ = app.cfg.save(); }
            "t" => { app.prompt_temp(); app.render_all(); let _ = app.cfg.save(); }
            "w" => { app.prompt_wind(); app.render_all(); let _ = app.cfg.save(); }
            "W" => { let _ = app.cfg.save(); app.set_status("Config saved", 46); app.render_status(); }
            _ => {}
        }
    }

    Crust::cleanup();
}

struct App {
    cfg: Config,
    days: Vec<DayForecast>,
    today: (i32, u32, u32),
    selected: (i32, u32, u32),
    cols: u16,
    rows: u16,
    top: Pane,
    left: Pane,
    right: Pane,
    status: Pane,
    status_msg: Option<(String, u8)>,
}

impl App {
    fn new(cfg: Config) -> Self {
        let (cols, rows) = Crust::terminal_size();
        let today = date_util::today();
        let (top, left, right, status) = Self::build_panes(cols, rows);
        Self {
            cfg,
            days: Vec::new(),
            today,
            selected: today,
            cols,
            rows,
            top,
            left,
            right,
            status,
            status_msg: None,
        }
    }

    fn build_panes(cols: u16, rows: u16) -> (Pane, Pane, Pane, Pane) {
        let left_w = cols / 3;
        let right_w = cols - left_w - 1;
        let content_h = rows.saturating_sub(2);
        let mut top = Pane::new(1, 1, cols, 1, 0, 208);
        top.wrap = false;
        let mut left = Pane::new(1, 2, left_w, content_h, 252, 0);
        left.wrap = false;
        left.border = true;
        left.border_fg = Some(238);
        let mut right = Pane::new(left_w + 2, 2, right_w, content_h, 252, 0);
        right.wrap = false;
        right.border = true;
        right.border_fg = Some(238);
        let status = Pane::new(1, rows, cols, 1, 245, 236);
        (top, left, right, status)
    }

    fn tick(&mut self) {
        // Keep today fresh if the clock crossed midnight.
        let now = date_util::today();
        if now != self.today {
            self.today = now;
            self.render_all();
        }
    }

    fn refresh_weather(&mut self, force: bool) {
        self.set_status("Fetching weather...", 226);
        self.render_status();
        self.days = weather::fetch_cached(self.cfg.lat, self.cfg.lon, force);
        if self.days.is_empty() {
            self.set_status("Weather fetch failed", 196);
        } else {
            self.set_status(&format!("{} days cached", self.days.len()), 46);
        }
    }

    fn move_day(&mut self, n: i32) {
        self.selected = date_util::add_days(self.selected, n);
    }

    fn go_today(&mut self) {
        self.selected = self.today;
    }

    fn render_all(&mut self) {
        Crust::clear_screen();
        self.render_top();
        self.render_left();
        self.render_right();
        self.render_status();
    }

    fn render_top(&mut self) {
        let label = format!(" nova  \u{2022} {}  (lat {:.2}, lon {:.2})",
            self.cfg.location, self.cfg.lat, self.cfg.lon);
        let right = format!("q:Quit  ?:Help  j/k:Day  r/R:Refresh  l:Loc  c/h/t/w:Limits  W:Save ");
        let pad = (self.cols as usize).saturating_sub(
            crust::display_width(&label) + crust::display_width(&right));
        self.top.say(&format!("{}{}{}", label, " ".repeat(pad), right));
    }

    fn render_left(&mut self) {
        let mut lines = Vec::new();
        lines.push(style::bold(&style::fg("9-Day Forecast", 81)));
        lines.push(String::new());

        let start = self.today;
        for i in 0..10 {
            let date = date_util::add_days(start, i);
            let ymd = date_util::format_ymd(date);
            let wd = date_util::weekday(date.0, date.1, date.2);
            let day = self.days.iter().find(|d| d.date == ymd);
            let marker = if date == self.selected { "\u{2192} " } else { "  " };
            let is_today = date == self.today;

            let header = format!("{}{} {} {}",
                marker,
                date_util::weekday_short(wd),
                date_util::month_short(date.1),
                date.2);

            let header = if is_today {
                style::bold(&style::fg(&header, 255))
            } else if date == self.selected {
                style::bold(&header)
            } else {
                header
            };

            match day {
                Some(d) => {
                    let points = weather::condition_points(
                        d.cloud, d.humidity, d.temp_mid, d.wind,
                        self.cfg.cloud_limit, self.cfg.humidity_limit,
                        self.cfg.temp_limit, self.cfg.wind_limit);
                    let color = weather::condition_color(points);
                    let summary = format!("  {} {:>4.1}° cld{:>3}% wind{:>3.1} hum{:>3.0}%",
                        d.symbol, d.temp_mid, d.cloud, d.wind, d.humidity);
                    lines.push(header);
                    lines.push(style::fg(&summary, color));
                }
                None => {
                    lines.push(header);
                    lines.push(style::fg("  (no forecast)", 240));
                }
            }
            lines.push(String::new());
        }

        self.left.set_text(&lines.join("\n"));
        self.left.ix = 0;
        self.left.full_refresh();
        if self.left.border { self.left.border_refresh(); }
    }

    fn render_right(&mut self) {
        let (y, m, d) = self.selected;
        let ymd = date_util::format_ymd(self.selected);
        let wd = date_util::weekday(y, m, d);

        let mut lines = Vec::new();
        lines.push(style::bold(&style::fg(
            &format!("{} {} {} {}",
                date_util::weekday_short(wd),
                date_util::month_short(m),
                d, y),
            226)));
        lines.push(String::new());

        // Sun
        if let Some((rise, set)) = astronomy::sun_times(y, m, d, self.cfg.lat, self.cfg.lon, self.cfg.tz) {
            lines.push(format!("{} Sun:  rise {}  set {}",
                style::fg("\u{2600}", 226), rise, set));
        }

        // Moon
        let mph = astronomy::moon_phase(y, m, d);
        if let Some((rise, set)) = astronomy::moon_times(y, m, d, self.cfg.lat, self.cfg.lon, self.cfg.tz) {
            lines.push(format!("{} Moon: rise {}  set {}   {} {} ({:.0}%)",
                style::fg(astronomy::moon_symbol(y, m, d), 159),
                rise, set, mph.phase_name, mph.symbol, mph.illumination * 100.0));
        } else {
            lines.push(format!("{} Moon: {} {} ({:.0}%)",
                style::fg(astronomy::moon_symbol(y, m, d), 159),
                mph.phase_name, mph.symbol, mph.illumination * 100.0));
        }
        lines.push(String::new());

        // Planets
        if self.cfg.show_planets {
            lines.push(style::bold(&style::fg("Planets:", 117)));
            let planets = astronomy::visible_planets(y, m, d, self.cfg.lat, self.cfg.lon, self.cfg.tz);
            if planets.is_empty() {
                lines.push(style::fg("  (none visible)", 240));
            } else {
                for p in planets {
                    let color = match p.color {
                        "yellow" => 226, "orange" => 208, "red" => 196,
                        "cyan" => 81, "blue" => 33, "white" => 255,
                        "gold" => 220, _ => 252,
                    };
                    lines.push(format!("  {} {:<8} rise {}  set {}",
                        style::fg(p.symbol, color),
                        style::fg(p.name, color),
                        p.rise, p.set));
                }
            }
            lines.push(String::new());
        }

        // Weather detail
        if let Some(day) = self.days.iter().find(|x| x.date == ymd) {
            lines.push(style::bold(&style::fg("Weather:", 117)));
            let points = weather::condition_points(
                day.cloud, day.humidity, day.temp_mid, day.wind,
                self.cfg.cloud_limit, self.cfg.humidity_limit,
                self.cfg.temp_limit, self.cfg.wind_limit);
            let label = if points >= 4 { "\u{25CF} BAD" }
                else if points >= 2 { "\u{25CF} FAIR" }
                else { "\u{25CF} GOOD" };
            let color = weather::condition_color(points);
            lines.push(format!("  Condition: {}", style::bold(&style::fg(label, color))));
            lines.push(format!("  Temp:      {}° (high {}° low {}°)",
                day.temp_mid, day.temp_high, day.temp_low));
            lines.push(format!("  Cloud:     {}%", day.cloud));
            lines.push(format!("  Wind:      {} m/s", day.wind));
            lines.push(format!("  Humidity:  {}%", day.humidity));
            lines.push(String::new());

            if !day.hours.is_empty() {
                lines.push(style::bold(&style::fg("Hours (3h):", 117)));
                for h in day.hours.iter().step_by(3).take(8) {
                    lines.push(format!("  {:02}:00  {:>5.1}°  cld{:>3}%  wind{:>3.1}  hum{:>3.0}%",
                        h.hour, h.temp, h.cloud, h.wind, h.humidity));
                }
                lines.push(String::new());
            }
        }

        // Astronomical events
        if self.cfg.show_events {
            let events = astronomy::astro_events_for_year(y, m, d);
            if !events.is_empty() {
                lines.push(style::bold(&style::fg("Astronomical events:", 117)));
                for e in events {
                    lines.push(format!("  {} {}", style::fg("\u{2022}", 220), e));
                }
            }
        }

        self.right.set_text(&lines.join("\n"));
        self.right.ix = 0;
        self.right.full_refresh();
        if self.right.border { self.right.border_refresh(); }
    }

    fn set_status(&mut self, msg: &str, color: u8) {
        self.status_msg = Some((msg.to_string(), color));
    }

    fn render_status(&mut self) {
        let msg = match &self.status_msg {
            Some((m, c)) => style::fg(m, *c),
            None => style::fg("Ready", 245),
        };
        self.status.say(&format!(" {}", msg));
    }

    fn prompt(&mut self, label: &str, default: &str) -> String {
        self.status.ix = 0;
        self.status.ask(label, default)
    }

    fn prompt_location(&mut self) {
        let input = self.prompt("Location (name lat,lon): ",
            &format!("{} {},{}", self.cfg.location, self.cfg.lat, self.cfg.lon));
        let trimmed = input.trim();
        if trimmed.is_empty() { return; }
        // Parse "name lat,lon" or "lat,lon"
        if let Some((name_part, coord_part)) = parse_loc(trimmed) {
            self.cfg.location = name_part;
            self.cfg.lat = coord_part.0;
            self.cfg.lon = coord_part.1;
            self.refresh_weather(true);
        } else {
            self.set_status("Bad format (expected 'Name 59.91,10.75')", 196);
        }
    }

    fn prompt_cloud(&mut self) {
        let s = self.prompt("Cloud limit %: ", &self.cfg.cloud_limit.to_string());
        if let Ok(n) = s.trim().parse::<i64>() { self.cfg.cloud_limit = n; }
    }

    fn prompt_humidity(&mut self) {
        let s = self.prompt("Humidity limit %: ", &self.cfg.humidity_limit.to_string());
        if let Ok(n) = s.trim().parse::<f64>() { self.cfg.humidity_limit = n; }
    }

    fn prompt_temp(&mut self) {
        let s = self.prompt("Temp lower limit °C: ", &self.cfg.temp_limit.to_string());
        if let Ok(n) = s.trim().parse::<f64>() { self.cfg.temp_limit = n; }
    }

    fn prompt_wind(&mut self) {
        let s = self.prompt("Wind limit m/s: ", &self.cfg.wind_limit.to_string());
        if let Ok(n) = s.trim().parse::<f64>() { self.cfg.wind_limit = n; }
    }

    fn show_help(&mut self) {
        let help = vec![
            style::bold(&style::fg("nova - Terminal astronomy panel", 226)),
            String::new(),
            style::bold("Keys:"),
            "  j / DOWN     Next day".into(),
            "  k / UP       Previous day".into(),
            "  0 / HOME     Jump to today".into(),
            "  r            Reload weather (from cache if fresh)".into(),
            "  R            Force re-fetch weather".into(),
            "  l            Set location (Name lat,lon)".into(),
            "  c / h / t / w  Cloud / Humidity / Temp / Wind limits".into(),
            "  W            Save config".into(),
            "  ? / q / ESC  Help / Quit / Close".into(),
            String::new(),
            style::fg("Config: ~/.nova/config.yml", 240),
            style::fg("Cache:  ~/.nova/weather_cache.json", 240),
            String::new(),
            style::fg("Press any key to close.", 240),
        ];
        self.right.set_text(&help.join("\n"));
        self.right.ix = 0;
        self.right.full_refresh();
        if self.right.border { self.right.border_refresh(); }
        let _ = Input::getchr(None);
        self.render_right();
    }
}

fn parse_loc(s: &str) -> Option<(String, (f64, f64))> {
    // Accept "Name lat,lon" or "lat,lon" (name falls back to current).
    let (name, rest) = match s.rsplit_once(' ') {
        Some((n, r)) if r.contains(',') => (n.trim().to_string(), r),
        _ => (String::new(), s),
    };
    let (la, lo) = rest.split_once(',')?;
    let lat: f64 = la.trim().parse().ok()?;
    let lon: f64 = lo.trim().parse().ok()?;
    let name = if name.is_empty() { format!("{:.2},{:.2}", lat, lon) } else { name };
    Some((name, (lat, lon)))
}
