#!/usr/bin/env rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rayon::prelude::*;

// Define style structure for grid cells
struct StyleConfig {
    name: &'static str,
    alive_str: &'static str,
    dead_str: &'static str,
    bg_color: Color,
    color: Color,
    is_solid: bool,
}

// Predefined themes (Catppuccin Mocha, Tokyo Night, Cyberpunk, Gruvbox, Nord, Monokai)
const STYLES: &[StyleConfig] = &[
    StyleConfig {
        name: "Mocha Green",
        alive_str: "■ ",
        dead_str: "· ",
        bg_color: Color::Rgb(30, 30, 46), // base_bg
        color: Color::Rgb(166, 227, 161),  // green
        is_solid: false,
    },
    StyleConfig {
        name: "Mocha Lavender",
        alive_str: "● ",
        dead_str: "· ",
        bg_color: Color::Rgb(30, 30, 46),
        color: Color::Rgb(180, 190, 254), // lavender
        is_solid: false,
    },
    StyleConfig {
        name: "Mocha Teal",
        alive_str: "  ",
        dead_str: "  ",
        bg_color: Color::Rgb(30, 30, 46),
        color: Color::Rgb(148, 226, 213), // teal
        is_solid: true,
    },
    StyleConfig {
        name: "Tokyo Night",
        alive_str: "  ",
        dead_str: "  ",
        bg_color: Color::Rgb(26, 27, 38),
        color: Color::Rgb(0, 240, 200),
        is_solid: true,
    },
    StyleConfig {
        name: "Cyberpunk",
        alive_str: "● ",
        dead_str: "· ",
        bg_color: Color::Reset,
        color: Color::Rgb(255, 50, 150),
        is_solid: false,
    },
    StyleConfig {
        name: "Gruvbox Retro",
        alive_str: "■ ",
        dead_str: "· ",
        bg_color: Color::Rgb(40, 40, 40),
        color: Color::Rgb(250, 189, 47), // yellow
        is_solid: false,
    },
    StyleConfig {
        name: "Nord Frost",
        alive_str: "● ",
        dead_str: "· ",
        bg_color: Color::Rgb(46, 52, 64),
        color: Color::Rgb(143, 188, 187), // frost cyan
        is_solid: false,
    },
    StyleConfig {
        name: "Monokai Bright",
        alive_str: "  ",
        dead_str: "  ",
        bg_color: Color::Rgb(39, 40, 34),
        color: Color::Rgb(249, 38, 114), // pink
        is_solid: true,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridMode {
    Square,
    Braille,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphMode {
    Rolling,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivePopup {
    None,
    Help,
    RulesEditor,
    ThemeManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RulesFocus {
    PresetList,
    TotalisticMatrix,
    NeighborhoodMask,
    DensitySlider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeFocus {
    PresetList,
    FgRed,
    FgGreen,
    FgBlue,
    BgRed,
    BgGreen,
    BgBlue,
    CustomCheckbox,
}

fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (128, 0, 0),
        Color::Green => (0, 128, 0),
        Color::Yellow => (128, 128, 0),
        Color::Blue => (0, 0, 128),
        Color::Magenta => (128, 0, 128),
        Color::Cyan => (0, 128, 128),
        Color::Gray => (192, 192, 192),
        Color::DarkGray => (80, 80, 80),
        Color::LightRed => (255, 0, 0),
        Color::LightGreen => (0, 255, 0),
        Color::LightYellow => (255, 255, 0),
        Color::LightBlue => (0, 0, 255),
        Color::LightMagenta => (255, 0, 255),
        Color::LightCyan => (0, 255, 255),
        Color::White => (255, 255, 255),
        _ => (30, 30, 46), // Default Base für Reset/Indexed
    }
}

struct RulesetPreset {
    name: &'static str,
    birth: [bool; 9],
    survive: [bool; 9],
}

const PRESET_RULESETS: &[RulesetPreset] = &[
    RulesetPreset {
        name: "Conway's Life (B3/S23)",
        birth: [false, false, false, true, false, false, false, false, false],
        survive: [false, false, true, true, false, false, false, false, false],
    },
    RulesetPreset {
        name: "HighLife (B36/S23)",
        birth: [false, false, false, true, false, false, true, false, false],
        survive: [false, false, true, true, false, false, false, false, false],
    },
    RulesetPreset {
        name: "Seeds (B2/S)",
        birth: [false, false, true, false, false, false, false, false, false],
        survive: [false, false, false, false, false, false, false, false, false],
    },
    RulesetPreset {
        name: "Replicator (B1357/S1357)",
        birth: [false, true, false, true, false, true, false, true, false],
        survive: [false, true, false, true, false, true, false, true, false],
    },
    RulesetPreset {
        name: "Day & Night (B3678/S34678)",
        birth: [false, false, false, true, false, false, true, true, true],
        survive: [false, false, false, true, true, false, true, true, true],
    },
    RulesetPreset {
        name: "Life Without Death (B3/S012345678)",
        birth: [false, false, false, true, false, false, false, false, false],
        survive: [true, true, true, true, true, true, true, true, true],
    },
    RulesetPreset {
        name: "Diamoeba (B35678/S5678)",
        birth: [false, false, false, true, false, true, true, true, true],
        survive: [false, false, false, false, false, true, true, true, true],
    },
    RulesetPreset {
        name: "Morley (B368/S245)",
        birth: [false, false, false, true, false, false, true, false, true],
        survive: [false, false, true, false, true, true, false, false, false],
    },
];

struct Stats {
    mode: GridMode,
    graph_mode: GraphMode,
    generation: usize,
    alive: usize,
    total: usize,
    max_alive: usize,
    min_alive: usize,
    density: f64,
    growth: i32,
    paused: bool,
    delay: std::time::Duration,
    birth_rules: [bool; 9],
    survive_rules: [bool; 9],
    fps_actual: f64,
}

struct AppState {
    mode: GridMode,
    grid: Vec<bool>,
    w_sim: usize,
    h_sim: usize,
    generation: usize,
    pop_history: Vec<usize>,
    max_alive: usize,
    min_alive: usize,
    growth: i32,
    paused: bool,
    delay: std::time::Duration,
    show_dashboard: bool,
    graph_mode: GraphMode,
    style_idx: usize,
    step_triggered: bool,
    grid_w_last: usize,
    grid_h_last: usize,
    
    // Customization & Menus
    popup: ActivePopup,
    rules_preset_idx: usize,
    rules_custom_birth: [bool; 9],
    rules_custom_survive: [bool; 9],
    rules_focus: RulesFocus,
    rules_matrix_row: usize,
    rules_matrix_col: usize,
    rules_neighbor_mask: [bool; 8],
    rules_mask_idx: usize,
    random_density: f64,
    
    theme_custom_r: u8,
    theme_custom_g: u8,
    theme_custom_b: u8,
    theme_custom_bg_r: u8,
    theme_custom_bg_g: u8,
    theme_custom_bg_b: u8,
    theme_focus: ThemeFocus,
    use_custom_colors: bool,
    
    // Slide Animation
    dash_width_pct: f32,
    target_dash_width_pct: f32,
    
    // Active ruleset
    birth_rules: [bool; 9],
    survive_rules: [bool; 9],
    
    // FPS Measurement
    fps_tick_count: usize,
    fps_last_calc: std::time::Instant,
    fps_actual: f64,
}

impl AppState {
    fn new() -> Self {
        Self {
            mode: GridMode::Square,
            grid: Vec::new(),
            w_sim: 0,
            h_sim: 0,
            generation: 0,
            pop_history: Vec::new(),
            max_alive: 0,
            min_alive: 0,
            growth: 0,
            paused: false,
            delay: std::time::Duration::from_millis(80),
            show_dashboard: true,
            graph_mode: GraphMode::Rolling,
            style_idx: 0,
            step_triggered: false,
            grid_w_last: 0,
            grid_h_last: 0,
            
            popup: ActivePopup::None,
            rules_preset_idx: 0,
            rules_custom_birth: [false, false, false, true, false, false, false, false, false], // B3
            rules_custom_survive: [false, false, true, true, false, false, false, false, false], // S23
            rules_focus: RulesFocus::PresetList,
            rules_matrix_row: 0,
            rules_matrix_col: 0,
            rules_neighbor_mask: [true; 8],
            rules_mask_idx: 0,
            random_density: 0.22,
            
            theme_custom_r: 166,
            theme_custom_g: 227,
            theme_custom_b: 161,
            theme_custom_bg_r: 30,
            theme_custom_bg_g: 30,
            theme_custom_bg_b: 46,
            theme_focus: ThemeFocus::PresetList,
            use_custom_colors: false,
            
            dash_width_pct: 1.0,
            target_dash_width_pct: 1.0,
            
            birth_rules: [false, false, false, true, false, false, false, false, false], // B3
            survive_rules: [false, false, true, true, false, false, false, false, false], // S23
            
            fps_tick_count: 0,
            fps_last_calc: std::time::Instant::now(),
            fps_actual: 0.0,
        }
    }

    fn alive_count(&self) -> usize {
        self.grid.iter().filter(|&&cell| cell).count()
    }

    fn change_mode(&mut self) {
        self.mode = match self.mode {
            GridMode::Square => GridMode::Braille,
            GridMode::Braille => GridMode::Square,
        };
        self.resize_grid(self.grid_w_last, self.grid_h_last, true);
    }

    fn check_resize(&mut self, term_w: usize, term_h: usize) {
        let (expected_w, expected_h) = match self.mode {
            GridMode::Braille => (term_w * 2, term_h * 4),
            GridMode::Square => (term_w / 2, term_h),
        };
        if self.w_sim != expected_w || self.h_sim != expected_h {
            let reinit = self.w_sim == 0 || self.h_sim == 0;
            self.resize_grid(term_w, term_h, reinit);
        }
    }

    fn resize_grid(&mut self, term_w: usize, term_h: usize, reinit: bool) {
        let (new_w, new_h) = match self.mode {
            GridMode::Braille => (term_w * 2, term_h * 4),
            GridMode::Square => (term_w / 2, term_h),
        };
        
        if reinit {
            self.grid = (0..(new_w * new_h))
                .map(|_| rand::random::<f64>() < self.random_density)
                .collect();
            self.w_sim = new_w;
            self.h_sim = new_h;
            self.generation = 0;
            let alive = self.alive_count();
            self.max_alive = alive;
            self.min_alive = alive;
            self.pop_history = vec![alive];
            self.growth = 0;
        } else {
            let mut new_grid = vec![false; new_w * new_h];
            for y in 0..self.h_sim.min(new_h) {
                for x in 0..self.w_sim.min(new_w) {
                    new_grid[y * new_w + x] = self.grid[y * self.w_sim + x];
                }
            }
            self.grid = new_grid;
            self.w_sim = new_w;
            self.h_sim = new_h;
        }
        self.grid_w_last = term_w;
        self.grid_h_last = term_h;
    }

    fn update_animation(&mut self) {
        let diff = self.target_dash_width_pct - self.dash_width_pct;
        if diff.abs() > 0.01 {
            self.dash_width_pct += diff * 0.25;
            if (self.dash_width_pct - self.target_dash_width_pct).abs() < 0.01 {
                self.dash_width_pct = self.target_dash_width_pct;
            }
        }
    }

    fn is_animating(&self) -> bool {
        (self.dash_width_pct - self.target_dash_width_pct).abs() > 0.001
    }

    fn tick(&mut self) {
        let step_triggered = self.step_triggered;
        self.step_triggered = false;
        
        let elapsed = self.fps_last_calc.elapsed();
        if elapsed >= std::time::Duration::from_secs(1) {
            self.fps_actual = self.fps_tick_count as f64 / elapsed.as_secs_f64();
            self.fps_tick_count = 0;
            self.fps_last_calc = std::time::Instant::now();
        }
        
        if !self.paused || step_triggered {
            let prev_alive = self.alive_count();
            self.grid = update_grid(&self.grid, self.w_sim, self.h_sim, &self.birth_rules, &self.survive_rules, &self.rules_neighbor_mask);
            self.generation += 1;
            
            self.fps_tick_count += 1;
            
            let alive = self.alive_count();
            self.growth = (alive as i32) - (prev_alive as i32);
            
            if self.generation == 1 {
                self.min_alive = alive;
                self.max_alive = alive.max(self.max_alive);
            } else {
                self.max_alive = self.max_alive.max(alive);
                self.min_alive = self.min_alive.min(alive);
            }
            self.pop_history.push(alive);
        }
        
        self.update_animation();
    }
}

fn update_grid(grid: &[bool], w: usize, h: usize, birth: &[bool; 9], survive: &[bool; 9], mask: &[bool; 8]) -> Vec<bool> {
    let mut new_grid = vec![false; w * h];
    new_grid.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let y_offset = y * w;
        let ym = ((y + h - 1) % h) * w;
        let yp = ((y + 1) % h) * w;
        for x in 0..w {
            let xm = (x + w - 1) % w;
            let xp = (x + 1) % w;
            
            let mut n = 0;
            if mask[0] && grid[ym + xm] { n += 1; }
            if mask[1] && grid[ym + x] { n += 1; }
            if mask[2] && grid[ym + xp] { n += 1; }
            if mask[3] && grid[y_offset + xm] { n += 1; }
            if mask[4] && grid[y_offset + xp] { n += 1; }
            if mask[5] && grid[yp + xm] { n += 1; }
            if mask[6] && grid[yp + x] { n += 1; }
            if mask[7] && grid[yp + xp] { n += 1; }
            
            let idx = y_offset + x;
            if grid[idx] {
                row[x] = survive[n];
            } else {
                row[x] = birth[n];
            }
        }
    });
    new_grid
}

fn set_pixel(canvas: &mut [u8], px_x: i32, px_y: i32, w: usize, h: usize) {
    if px_x >= 0 && px_x < w as i32 && px_y >= 0 && px_y < h as i32 {
        let grid_y = (h - 1) - px_y as usize;
        canvas[grid_y * w + px_x as usize] = 1;
    }
}

fn draw_line(canvas: &mut [u8], mut x0: i32, mut y0: i32, x1: i32, y1: i32, w: usize, h: usize) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    
    loop {
        set_pixel(canvas, x0, y0, w, h);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn get_braille_char(grid: &[bool], w: usize, h: usize, char_col: usize, char_row: usize) -> char {
    let mut val = 0;
    let base_x = char_col * 2;
    let base_y = char_row * 4;
    
    if base_y + 0 < h {
        if base_x + 0 < w && grid[(base_y + 0) * w + (base_x + 0)] { val |= 0x01; }
        if base_x + 1 < w && grid[(base_y + 0) * w + (base_x + 1)] { val |= 0x08; }
    }
    if base_y + 1 < h {
        if base_x + 0 < w && grid[(base_y + 1) * w + (base_x + 0)] { val |= 0x02; }
        if base_x + 1 < w && grid[(base_y + 1) * w + (base_x + 1)] { val |= 0x10; }
    }
    if base_y + 2 < h {
        if base_x + 0 < w && grid[(base_y + 2) * w + (base_x + 0)] { val |= 0x04; }
        if base_x + 1 < w && grid[(base_y + 2) * w + (base_x + 1)] { val |= 0x20; }
    }
    if base_y + 3 < h {
        if base_x + 0 < w && grid[(base_y + 3) * w + (base_x + 0)] { val |= 0x40; }
        if base_x + 1 < w && grid[(base_y + 3) * w + (base_x + 1)] { val |= 0x80; }
    }
    
    std::char::from_u32(0x2800 + val).unwrap_or(' ')
}

fn render_dashboard(
    w_char: usize,
    h_char: usize,
    stats: &Stats,
    history: &[usize],
    active_style: &StyleConfig,
    active_cell_color: Color,
    active_bg_color: Color,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let bg_color = active_bg_color;
    
    let pad_spans = |spans: Vec<Span<'static>>| -> Line<'static> {
        let vis_len = spans.iter().map(|s| s.width()).sum::<usize>();
        let mut spans = spans;
        if vis_len < w_char {
            spans.push(Span::styled(" ".repeat(w_char - vis_len), Style::default().bg(bg_color)));
        }
        Line::from(spans)
    };
    
    let format_kv = |label: &'static str, val_styled: Span<'static>, val_raw: &str| -> Line<'static> {
        let vis_label = label.len();
        let vis_val = val_raw.len();
        let spaces = w_char.saturating_sub(2).saturating_sub(vis_label).saturating_sub(vis_val);
        let spaces = spaces.max(1);
        Line::from(vec![
            Span::styled(label, Style::default().fg(Color::Rgb(205, 214, 244))),
            Span::raw(" ".repeat(spaces)),
            val_styled,
        ])
    };
    
    // 1. Header Title
    let title = "⚡ DASHBOARD ⚡";
    let title_len = 15; // Emojis take 2 columns each
    let pad_left = w_char.saturating_sub(title_len) / 2;
    lines.push(pad_spans(vec![
        Span::raw(" ".repeat(pad_left)),
        Span::styled(title, Style::default().fg(Color::Rgb(203, 166, 247))), // mauve
    ]));
    
    lines.push(pad_spans(vec![
        Span::styled("-".repeat(w_char), Style::default().fg(Color::Rgb(88, 91, 112))), // surface
    ]));
    
    // 2. Stats
    let mode_str = if stats.mode == GridMode::Braille { "Braille (Hi-Res)" } else { "Square" };
    let mode_val = if stats.mode == GridMode::Braille {
        Span::styled("Braille (Hi-Res)", Style::default().fg(Color::Rgb(137, 220, 235))) // sky
    } else {
        Span::styled("Square", Style::default().fg(Color::Rgb(250, 179, 135))) // peach
    };
    lines.push(pad_spans(format_kv(" Grid Mode:", mode_val, mode_str).spans));
    
    let theme_name_short = if active_style.name.len() > 18 {
        &active_style.name[..18]
    } else {
        active_style.name
    };
    let theme_val = Span::styled(theme_name_short, Style::default().fg(active_cell_color));
    lines.push(pad_spans(format_kv(" Theme:", theme_val, theme_name_short).spans));
    
    let rules_str = format!(
        "B{}/S{}",
        stats.birth_rules.iter().enumerate().filter(|&(_, &b)| b).map(|(i, _)| i.to_string()).collect::<Vec<_>>().join(""),
        stats.survive_rules.iter().enumerate().filter(|&(_, &s)| s).map(|(i, _)| i.to_string()).collect::<Vec<_>>().join("")
    );
    let rules_val = Span::styled(rules_str.clone(), Style::default().fg(Color::Rgb(203, 166, 247))); // mauve
    lines.push(pad_spans(format_kv(" Ruleset:", rules_val, &rules_str).spans));
    
    lines.push(pad_spans(format_kv(
        " Generation:", 
        Span::styled(stats.generation.to_string(), Style::default().fg(Color::Rgb(249, 226, 175))), // yellow
        &stats.generation.to_string()
    ).spans));
    
    let alive_raw = format!("{} / {}", stats.alive, stats.total);
    let alive_spaces = w_char.saturating_sub(2).saturating_sub(" Alive Cells:".len()).saturating_sub(alive_raw.len()).max(1);
    lines.push(pad_spans(vec![
        Span::styled(" Alive Cells:", Style::default().fg(Color::Rgb(205, 214, 244))),
        Span::raw(" ".repeat(alive_spaces)),
        Span::styled(stats.alive.to_string(), Style::default().fg(Color::Rgb(166, 227, 161))), // green
        Span::styled(" / ", Style::default().fg(Color::Rgb(108, 112, 134))), // overlay
        Span::styled(stats.total.to_string(), Style::default().fg(Color::Rgb(108, 112, 134))),
    ]));
    
    lines.push(pad_spans(format_kv(
        " Max Alive:", 
        Span::styled(stats.max_alive.to_string(), Style::default().fg(Color::Rgb(250, 179, 135))), // peach
        &stats.max_alive.to_string()
    ).spans));
    
    lines.push(pad_spans(format_kv(
        " Min Alive:", 
        Span::styled(stats.min_alive.to_string(), Style::default().fg(Color::Rgb(137, 220, 235))), // sky
        &stats.min_alive.to_string()
    ).spans));
    
    let density_str = format!("{:.2}%", stats.density);
    lines.push(pad_spans(format_kv(
        " Density:", 
        Span::styled(density_str.clone(), Style::default().fg(Color::Rgb(148, 226, 213))), // teal
        &density_str
    ).spans));
    
    let (growth_span, growth_raw) = if stats.growth > 0 {
        (Span::styled(format!("+{}", stats.growth), Style::default().fg(Color::Rgb(166, 227, 161))), format!("+{}", stats.growth))
    } else if stats.growth < 0 {
        (Span::styled(stats.growth.to_string(), Style::default().fg(Color::Rgb(243, 139, 168))), stats.growth.to_string())
    } else {
        (Span::styled("0", Style::default().fg(Color::Rgb(166, 173, 200))), "0".to_string())
    };
    lines.push(pad_spans(format_kv(" Growth Rate:", growth_span, &growth_raw).spans));
    
    let status_str = if stats.paused { "Paused (Step)" } else { "Running" };
    let status_val = if stats.paused {
        Span::styled("Paused (Step)", Style::default().fg(Color::Rgb(243, 139, 168)))
    } else {
        Span::styled("Running", Style::default().fg(Color::Rgb(166, 227, 161)))
    };
    lines.push(pad_spans(format_kv(" Status:", status_val, status_str).spans));
    
    let gmode_str = if stats.graph_mode == GraphMode::Full { "Full History" } else { "Rolling" };
    let gmode_val = if stats.graph_mode == GraphMode::Full {
        Span::styled("Full History", Style::default().fg(Color::Rgb(137, 220, 235)))
    } else {
        Span::styled("Rolling", Style::default().fg(Color::Rgb(244, 184, 228)))
    };
    lines.push(pad_spans(format_kv(" Graph Mode:", gmode_val, gmode_str).spans));
    
    lines.push(pad_spans(vec![
        Span::styled("-".repeat(w_char), Style::default().fg(Color::Rgb(88, 91, 112))),
    ]));
    
    // 3. Braille Live Plot
    let g_title = if stats.graph_mode == GraphMode::Full { "📈 Live Pop. (Full)" } else { "📈 Live Pop. (Rolling)" };
    lines.push(pad_spans(vec![
        Span::styled(" ", Style::default()),
        Span::styled(g_title, Style::default().fg(Color::Rgb(244, 184, 228))), // pink
        Span::raw(":"),
    ]));
    
    let graph_h_char = (h_char as i32 - 23).max(4) as usize;
    let mut graph_w_char = w_char.saturating_sub(8);
    if graph_w_char < 10 {
        graph_w_char = 10;
    }
    
    let pw = graph_w_char * 2;
    let ph = graph_h_char * 4;
    let mut canvas = vec![0u8; pw * ph];
    
    if history.len() >= 2 {
        let start_gen;
        let end_gen;
        let hist_subset;
        if stats.graph_mode == GraphMode::Rolling || history.len() <= pw {
            hist_subset = history[history.len().saturating_sub(pw)..].to_vec();
            start_gen = history.len().saturating_sub(pw);
            end_gen = history.len() - 1;
        } else {
            let mut subset = Vec::with_capacity(pw);
            for i in 0..pw {
                let idx = (i * (history.len() - 1)) / (pw - 1);
                subset.push(history[idx]);
            }
            hist_subset = subset;
            start_gen = 0;
            end_gen = history.len() - 1;
        }
        
        let min_v = *hist_subset.iter().min().unwrap_or(&0);
        let max_v = *hist_subset.iter().max().unwrap_or(&0);
        let v_range = max_v - min_v;
        
        let mut points = Vec::with_capacity(hist_subset.len());
        for (i, &val) in hist_subset.iter().enumerate() {
            let x = i as i32;
            let y = if v_range == 0 {
                (ph / 2) as i32
            } else {
                ((val - min_v) as f64 / v_range as f64 * (ph - 1) as f64) as i32
            };
            points.push((x, y));
        }
        
        for i in 0..points.len().saturating_sub(1) {
            draw_line(&mut canvas, points[i].0, points[i].1, points[i+1].0, points[i+1].1, pw, ph);
        }
        
        for r in 0..graph_h_char {
            let mut row_chars = String::new();
            for c in 0..graph_w_char {
                let mut val = 0;
                let base_x = c * 2;
                let base_y = r * 4;
                
                if canvas[(base_y + 0) * pw + (base_x + 0)] != 0 { val |= 0x01; }
                if canvas[(base_y + 0) * pw + (base_x + 1)] != 0 { val |= 0x08; }
                if canvas[(base_y + 1) * pw + (base_x + 0)] != 0 { val |= 0x02; }
                if canvas[(base_y + 1) * pw + (base_x + 1)] != 0 { val |= 0x10; }
                if canvas[(base_y + 2) * pw + (base_x + 0)] != 0 { val |= 0x04; }
                if canvas[(base_y + 2) * pw + (base_x + 1)] != 0 { val |= 0x20; }
                if canvas[(base_y + 3) * pw + (base_x + 0)] != 0 { val |= 0x40; }
                if canvas[(base_y + 3) * pw + (base_x + 1)] != 0 { val |= 0x80; }
                
                row_chars.push(std::char::from_u32(0x2800 + val).unwrap_or(' '));
            }
            
            let y_label = if r == 0 {
                format!("{:5} ", max_v)
            } else if r == graph_h_char - 1 {
                format!("{:5} ", min_v)
            } else {
                "      ".to_string()
            };
            
            lines.push(pad_spans(vec![
                Span::styled(y_label, Style::default().fg(Color::Rgb(166, 173, 200))), // subtext
                Span::styled(row_chars, Style::default().fg(Color::Rgb(148, 226, 213))), // teal
            ]));
        }
        
        // Render x-axis labels
        let left_lbl = start_gen.to_string();
        let right_lbl = end_gen.to_string();
        let spaces_count = graph_w_char.saturating_sub(left_lbl.len()).saturating_sub(right_lbl.len());
        lines.push(pad_spans(vec![
            Span::raw(" ".repeat(7)),
            Span::styled(left_lbl, Style::default().fg(Color::Rgb(166, 173, 200))),
            Span::raw(" ".repeat(spaces_count)),
            Span::styled(right_lbl, Style::default().fg(Color::Rgb(166, 173, 200))),
        ]));
    } else {
        for _ in 0..graph_h_char {
            lines.push(pad_spans(vec![
                Span::raw("      "),
                Span::styled(".".repeat(graph_w_char), Style::default().fg(Color::Rgb(108, 112, 134))),
            ]));
        }
        
        let left_lbl = "0";
        let right_lbl = "0";
        let spaces_count = graph_w_char.saturating_sub(left_lbl.len()).saturating_sub(right_lbl.len());
        lines.push(pad_spans(vec![
            Span::raw(" ".repeat(7)),
            Span::styled(left_lbl, Style::default().fg(Color::Rgb(166, 173, 200))),
            Span::raw(" ".repeat(spaces_count)),
            Span::styled(right_lbl, Style::default().fg(Color::Rgb(166, 173, 200))),
        ]));
    }
    
    lines.push(pad_spans(vec![
        Span::styled("-".repeat(w_char), Style::default().fg(Color::Rgb(88, 91, 112))),
    ]));
    
    // 4. Legend info (vollständig und perfekt ausgerichtet!)
    lines.push(pad_spans(vec![
        Span::styled(" Keyboard Shortcuts:", Style::default().add_modifier(Modifier::BOLD).fg(Color::Rgb(249, 226, 175))),
    ]));
    
    let make_shortcut_line = |k1: &'static str, l1: &'static str, k2: &'static str, l2: &'static str, k3: &'static str, l3: &'static str| -> Vec<Span<'static>> {
        vec![
            Span::raw("  "),
            Span::styled(format!("{:<5}", k1), Style::default().fg(Color::Rgb(137, 180, 250))),
            Span::raw(" "),
            Span::styled(format!("{:<6}", l1), Style::default().fg(Color::Rgb(205, 214, 244))),
            Span::raw(" | "),
            Span::styled(format!("{:<2}", k2), Style::default().fg(Color::Rgb(137, 180, 250))),
            Span::raw(" "),
            Span::styled(format!("{:<5}", l2), Style::default().fg(Color::Rgb(205, 214, 244))),
            Span::raw(" | "),
            Span::styled(format!("{:<2}", k3), Style::default().fg(Color::Rgb(137, 180, 250))),
            Span::raw(" "),
            Span::styled(l3, Style::default().fg(Color::Rgb(205, 214, 244))),
        ]
    };

    lines.push(pad_spans(make_shortcut_line("Space", "Pause", "N", "Step", "D", "Dash")));
    lines.push(pad_spans(make_shortcut_line("M", "Grid M", "V", "Graph", "S", "Style")));
    lines.push(pad_spans(make_shortcut_line("R", "Random", "C", "Clear", "Q", "Quit")));
    lines.push(pad_spans(make_shortcut_line("E", "Rules", "T", "Theme", "H", "Help")));
    
    let target_fps_str = if stats.delay.as_millis() == 0 {
        "max".to_string()
    } else {
        format!("{:.1}", 1.0 / stats.delay.as_secs_f64())
    };
    let actual_fps_str = format!("{:.1}", stats.fps_actual);
    let fps_info = format!("tgt:{} act:{} fps", target_fps_str, actual_fps_str);
    
    lines.push(pad_spans(vec![
        Span::raw("  "),
        Span::styled(format!("{:<5}", "+/-"), Style::default().fg(Color::Rgb(137, 180, 250))),
        Span::raw(" "),
        Span::styled(format!("{:<6}", "Speed"), Style::default().fg(Color::Rgb(205, 214, 244))),
        Span::raw(" | "),
        Span::styled(fps_info, Style::default().fg(Color::Rgb(249, 226, 175))),
    ]));
    
    while lines.len() < h_char {
        lines.push(pad_spans(vec![]));
    }
    
    lines.truncate(h_char);
    lines
}

fn fixed_centered_rect(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let pad_x = r.width.saturating_sub(width) / 2;
    let pad_y = r.height.saturating_sub(height) / 2;
    
    ratatui::layout::Rect {
        x: r.x + pad_x,
        y: r.y + pad_y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

fn render_help_popup(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let popup_area = fixed_centered_rect(68, 19, area);
    f.render_widget(ratatui::widgets::Clear, popup_area);
    
    let block = Block::default()
        .title(" [ Info: Game of Life & TUI Help ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(137, 180, 250))) // blue
        .style(Style::default().bg(Color::Rgb(30, 30, 46))); // base
        
    let birth_str = state.birth_rules.iter().enumerate().filter(|&(_, &b)| b).map(|(i, _)| i.to_string()).collect::<Vec<_>>().join("");
    let survive_str = state.survive_rules.iter().enumerate().filter(|&(_, &s)| s).map(|(i, _)| i.to_string()).collect::<Vec<_>>().join("");
    let rules_desc = format!("B{}/S{}", birth_str, survive_str);
        
    let text = vec![
        Line::from(vec![
            Span::styled("Simulationregeln: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Rgb(249, 226, 175))),
            Span::styled(rules_desc, Style::default().fg(Color::Rgb(203, 166, 247))),
        ]),
        Line::from(""),
        Line::from("  1. Überleben: Eine lebende Zelle bleibt aktiv, wenn die Anzahl"),
        Line::from(format!("     ihrer Nachbarn in S ({}) enthalten ist.", if survive_str.is_empty() { "keine" } else { &survive_str })),
        Line::from("  2. Geburt: Eine inaktive Zelle wird aktiv, wenn die Anzahl"),
        Line::from(format!("     ihrer Nachbarn in B ({}) enthalten ist.", if birth_str.is_empty() { "keine" } else { &birth_str })),
        Line::from("  3. Tod: In allen anderen Fällen stirbt die Zelle."),
        Line::from(""),
        Line::from(Span::styled("Tastatur-Kurzbefehle:", Style::default().add_modifier(Modifier::BOLD).fg(Color::Rgb(249, 226, 175)))),
        Line::from("  [Space]   Pause / Fortsetzen (Simulation einfrieren)"),
        Line::from("  [N]/[Ent] Einzelschritt (Simulation tickt einmal)"),
        Line::from("  [D]       Dashboard Panel ein-/ausblenden (mit Animation!)"),
        Line::from("  [M]       Umschalten: Braille (Hi-Res) / Square (Zwei-Zeichen)"),
        Line::from("  [V]       Umschalten: Rolling Graph / Full History Graph"),
        Line::from("  [R]       Spielfeld zufällig neu besiedeln (im Editor auch!)"),
        Line::from("  [C]       Spielfeld leeren (Clear)"),
        Line::from("  [+/-]     Geschwindigkeit anpassen (bis zu 1000 FPS!)"),
        Line::from("  [E]       Rules Editor: Regeln, Nachbarschaft & Dichte"),
        Line::from("  [T]       Theme Manager: Farbschemata & Custom RGB-Editor"),
        Line::from("  [H]/[?]   Diese Hilfe anzeigen / schließen"),
        Line::from("  [Q]/[Esc] Beenden"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Rgb(205, 214, 244))); // text
        
    f.render_widget(paragraph, popup_area);
}

fn render_rules_popup(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let popup_area = fixed_centered_rect(72, 22, area);
    f.render_widget(ratatui::widgets::Clear, popup_area);
    
    let block = Block::default()
        .title(" [ Rules Editor & Simulator Options ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247))) // mauve
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));
        
    let mut lines = Vec::new();
    
    let preset_focused = state.rules_focus == RulesFocus::PresetList;
    lines.push(Line::from(vec![
        Span::styled("Regel-Presets: ", if preset_focused { Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Rgb(205, 214, 244)) }),
        Span::raw("(Pfeiltasten hoch/runter, Space zum Auswählen)"),
    ]));
    
    for i in 0..PRESET_RULESETS.len() {
        let preset = &PRESET_RULESETS[i];
        let is_selected = i == state.rules_preset_idx;
        let prefix = if is_selected && preset_focused {
            " > "
        } else if is_selected {
            " * "
        } else {
            "   "
        };
        let style = if is_selected && preset_focused {
            Style::default().fg(Color::Rgb(166, 227, 161)).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::Rgb(137, 180, 250))
        } else {
            Style::default().fg(Color::Rgb(166, 173, 200))
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(preset.name, style),
        ]));
    }
    
    lines.push(Line::from(""));
    
    let matrix_focused = state.rules_focus == RulesFocus::TotalisticMatrix;
    lines.push(Line::from(vec![
        Span::styled("Eigene B/S Matrix: ", if matrix_focused { Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Rgb(205, 214, 244)) }),
        Span::raw("(Pfeiltasten zum Navigieren, Space zum toggeln)"),
    ]));
    
    let mut birth_spans = vec![Span::raw("  Birth (B):   ")];
    for col in 0..9 {
        let val = state.rules_custom_birth[col];
        let is_focused = matrix_focused && state.rules_matrix_row == 0 && state.rules_matrix_col == col;
        
        let box_str = if val { "[x]" } else { "[ ]" };
        let box_style = if is_focused {
            Style::default().fg(Color::Rgb(249, 226, 175)).bg(Color::Rgb(69, 71, 90)).add_modifier(Modifier::BOLD)
        } else if val {
            Style::default().fg(Color::Rgb(166, 227, 161))
        } else {
            Style::default().fg(Color::Rgb(108, 112, 134))
        };
        birth_spans.push(Span::styled(format!("{}{}", box_str, col), box_style));
        birth_spans.push(Span::raw(" "));
    }
    lines.push(Line::from(birth_spans));
    
    let mut survive_spans = vec![Span::raw("  Survive (S): ")];
    for col in 0..9 {
        let val = state.rules_custom_survive[col];
        let is_focused = matrix_focused && state.rules_matrix_row == 1 && state.rules_matrix_col == col;
        
        let box_str = if val { "[x]" } else { "[ ]" };
        let box_style = if is_focused {
            Style::default().fg(Color::Rgb(249, 226, 175)).bg(Color::Rgb(69, 71, 90)).add_modifier(Modifier::BOLD)
        } else if val {
            Style::default().fg(Color::Rgb(166, 227, 161))
        } else {
            Style::default().fg(Color::Rgb(108, 112, 134))
        };
        survive_spans.push(Span::styled(format!("{}{}", box_str, col), box_style));
        survive_spans.push(Span::raw(" "));
    }
    lines.push(Line::from(survive_spans));
    
    lines.push(Line::from(""));
    
    let mask_focused = state.rules_focus == RulesFocus::NeighborhoodMask;
    lines.push(Line::from(vec![
        Span::styled("Nachbarschafts-Maske (Advanced): ", if mask_focused { Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Rgb(205, 214, 244)) }),
        Span::raw("(Space toggelt Nachbar)"),
    ]));
    
    let make_mask_box = |idx: usize, label: &str| -> Span<'static> {
        let val = state.rules_neighbor_mask[idx];
        let is_focused = mask_focused && state.rules_mask_idx == idx;
        let box_str = if val { "[x]" } else { "[ ]" };
        let box_style = if is_focused {
            Style::default().fg(Color::Rgb(249, 226, 175)).bg(Color::Rgb(69, 71, 90)).add_modifier(Modifier::BOLD)
        } else if val {
            Style::default().fg(Color::Rgb(166, 227, 161))
        } else {
            Style::default().fg(Color::Rgb(108, 112, 134))
        };
        Span::styled(format!("{}{}", box_str, label), box_style)
    };
    
    lines.push(Line::from(vec![
        Span::raw("  "),
        make_mask_box(0, "↖"), Span::raw(" "),
        make_mask_box(1, "↑"), Span::raw(" "),
        make_mask_box(2, "↗"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        make_mask_box(3, "←"), Span::raw(" "),
        Span::styled(" [C] ", Style::default().fg(Color::Rgb(203, 166, 247))), Span::raw(" "),
        make_mask_box(4, "→"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        make_mask_box(5, "↙"), Span::raw(" "),
        make_mask_box(6, "↓"), Span::raw(" "),
        make_mask_box(7, "↘"),
    ]));
    
    lines.push(Line::from(""));
    
    let density_focused = state.rules_focus == RulesFocus::DensitySlider;
    let density_pct = (state.random_density * 100.0) as usize;
    let bar_width = 30;
    let fill_chars = (density_pct * bar_width) / 100;
    let empty_chars = bar_width - fill_chars;
    let slider_bar = format!(
        "[{}{}{}]",
        "=".repeat(fill_chars),
        "|",
        " ".repeat(empty_chars)
    );
    let slider_style = if density_focused {
        Style::default().fg(Color::Rgb(249, 226, 175)).bg(Color::Rgb(69, 71, 90)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(148, 226, 213))
    };
    lines.push(Line::from(vec![
        Span::styled("  Random Density: ", if density_focused { Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Rgb(205, 214, 244)) }),
        Span::styled(slider_bar, slider_style),
        Span::styled(format!("  {}%", density_pct), if density_focused { Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Rgb(205, 214, 244)) }),
    ]));
    
    lines.push(Line::from(""));
    lines.push(Line::from("  [Tab] Bereich wechseln | [Pfeiltasten] Navigieren / Slider regeln"));
    lines.push(Line::from("  [Space] Togglen | [R] Spielfeld randomisieren | [E/Esc] Schließen"));
    
    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::Rgb(205, 214, 244)));
        
    f.render_widget(paragraph, popup_area);
}

fn render_theme_popup(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let popup_area = fixed_centered_rect(70, 22, area);
    f.render_widget(ratatui::widgets::Clear, popup_area);
    
    let block = Block::default()
        .title(" [ Theme Manager & Color Customizer ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(166, 227, 161))) // green
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));
        
    let mut lines = Vec::new();
    lines.push(Line::from("Farbschemata (Pfeiltasten hoch/runter):"));
    
    let list_focused = state.theme_focus == ThemeFocus::PresetList;
    for i in 0..STYLES.len() {
        let style_cfg = &STYLES[i];
        let is_selected = i == state.style_idx;
        let prefix = if is_selected && list_focused {
            " > "
        } else if is_selected {
            " * "
        } else {
            "   "
        };
        let style = if is_selected && list_focused {
            Style::default().fg(Color::Rgb(166, 227, 161)).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::Rgb(137, 180, 250))
        } else {
            Style::default().fg(Color::Rgb(166, 173, 200))
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(style_cfg.name, style),
        ]));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from("RGB Custom Farbeditor (Pfeiltasten links/rechts zum Ändern):"));
    lines.push(Line::from(""));
    
    let r_val = state.theme_custom_r;
    let g_val = state.theme_custom_g;
    let b_val = state.theme_custom_b;
    let bg_r = state.theme_custom_bg_r;
    let bg_g = state.theme_custom_bg_g;
    let bg_b = state.theme_custom_bg_b;
    
    let make_slider = |label: &str, val: u8, focus_target: ThemeFocus| -> Line<'static> {
        let is_focused = state.theme_focus == focus_target;
        let bar_width = 25;
        let fill_chars = (val as f32 / 255.0 * bar_width as f32) as usize;
        let empty_chars = bar_width - fill_chars;
        
        let slider_bar = format!(
            "[{}{}{}]",
            "=".repeat(fill_chars),
            "|",
            " ".repeat(empty_chars)
        );
        
        let label_style = if is_focused {
            Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(205, 214, 244))
        };
        
        let bar_color = match focus_target {
            ThemeFocus::FgRed => Color::Rgb(243, 139, 168),
            ThemeFocus::FgGreen => Color::Rgb(166, 227, 161),
            ThemeFocus::FgBlue => Color::Rgb(137, 180, 250),
            ThemeFocus::BgRed => Color::Rgb(243, 139, 168),
            ThemeFocus::BgGreen => Color::Rgb(166, 227, 161),
            ThemeFocus::BgBlue => Color::Rgb(137, 180, 250),
            _ => Color::Rgb(205, 214, 244),
        };
        
        let bar_style = if is_focused {
            Style::default().fg(bar_color).bg(Color::Rgb(69, 71, 90))
        } else {
            Style::default().fg(bar_color)
        };
        
        Line::from(vec![
            Span::styled(format!("  {: <10}", label), label_style),
            Span::styled(slider_bar, bar_style),
            Span::styled(format!("  {: >3}", val), label_style),
        ])
    };
    
    lines.push(make_slider("FG Red:", r_val, ThemeFocus::FgRed));
    lines.push(make_slider("FG Green:", g_val, ThemeFocus::FgGreen));
    lines.push(make_slider("FG Blue:", b_val, ThemeFocus::FgBlue));
    lines.push(make_slider("BG Red:", bg_r, ThemeFocus::BgRed));
    lines.push(make_slider("BG Green:", bg_g, ThemeFocus::BgGreen));
    lines.push(make_slider("BG Blue:", bg_b, ThemeFocus::BgBlue));
    
    lines.push(Line::from(""));
    
    // Checkbox rendern
    let checkbox_focused = state.theme_focus == ThemeFocus::CustomCheckbox;
    let box_str = if state.use_custom_colors { "[x]" } else { "[ ]" };
    let cb_style = if checkbox_focused {
        Style::default().fg(Color::Rgb(249, 226, 175)).bg(Color::Rgb(69, 71, 90)).add_modifier(Modifier::BOLD)
    } else if state.use_custom_colors {
        Style::default().fg(Color::Rgb(166, 227, 161))
    } else {
        Style::default().fg(Color::Rgb(166, 173, 200))
    };
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{} Eigene Farben verwenden (Checkbox mit Space toggeln)", box_str), cb_style),
    ]));
    
    lines.push(Line::from(""));
    
    let preview_fg = Color::Rgb(r_val, g_val, b_val);
    let preview_bg = Color::Rgb(bg_r, bg_g, bg_b);
    let preview_style = Style::default().fg(preview_fg).bg(preview_bg);
    let preview_span = if state.use_custom_colors {
        Span::styled("  [ AKTIV: Custom Farben ]", Style::default().fg(Color::Rgb(166, 227, 161)).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  [ Preset aktiv - custom inaktiv ]", Style::default().fg(Color::Rgb(166, 173, 200)))
    };
    
    lines.push(Line::from(vec![
        preview_span,
        Span::raw(" | Vorschau-Zelle: "),
        Span::styled("■●■ ", preview_style),
    ]));
    
    lines.push(Line::from(""));
    lines.push(Line::from("  [Tab/Pfeiltasten] Navigieren | [Pfeil Links/Rechts] Slider regeln"));
    lines.push(Line::from("  [Space] Togglen / Reset auf Preset | [T/Esc] Schließen"));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::Rgb(205, 214, 244)));
        
    f.render_widget(paragraph, popup_area);
}

fn make_status_text(paused: bool, generation: usize, alive: usize, delay: std::time::Duration, fps_actual: f64, show_dashboard: bool, term_w: usize) -> String {
    let pause_status = if paused { "Yes" } else { "No" };
    let target_fps_str = if delay.as_millis() == 0 {
        "max".to_string()
    } else {
        format!("{:.1}", 1.0 / delay.as_secs_f64())
    };
    let actual_fps_str = format!("{:.1}", fps_actual);
    
    if show_dashboard {
        let text = format!(" Pause: {} | Gen: {} | Alive: {} | Speed: tgt:{} act:{} fps", pause_status, generation, alive, target_fps_str, actual_fps_str);
        return format!("{: <width$}", text, width = term_w.saturating_sub(1));
    }
    
    let full = format!(
        " [Space] Pause: {} | Gen: {} | Alive: {} | tgt:{} act:{} fps | [E] Rules | [T] Theme | [H] Help | [D] Dash",
        pause_status, generation, alive, target_fps_str, actual_fps_str
    );
    if full.chars().count() <= term_w.saturating_sub(2) {
        return format!("{: <width$}", full, width = term_w.saturating_sub(1));
    }
    
    let med = format!(
        " Gen: {} | Alive: {} | tgt:{} act:{} | [E] Rls | [T] Thm | [H] Hlp | [D] Dsh",
        generation, alive, target_fps_str, actual_fps_str
    );
    if med.chars().count() <= term_w.saturating_sub(2) {
        return format!("{: <width$}", med, width = term_w.saturating_sub(1));
    }
    
    let short = format!(" G:{} A:{} tgt:{} act:{}", generation, alive, target_fps_str, actual_fps_str);
    format!("{: <width$}", short, width = term_w.saturating_sub(1))
}

fn draw_ui(f: &mut Frame, state: &mut AppState) {
    let area = f.area();
    
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);
    let main_area = vertical_chunks[0];
    let status_area = vertical_chunks[1];
    
    let (horizontal_chunks, current_dash_w) = {
        let mut max_dash_w = ((main_area.width as f32) * 0.33) as u16;
        max_dash_w = max_dash_w.clamp(36, 48).min(main_area.width.saturating_sub(4));
        
        let current_dash_w = (max_dash_w as f32 * state.dash_width_pct) as u16;
        
        let chunks = if current_dash_w > 0 {
            let mut grid_w = main_area.width.saturating_sub(current_dash_w).saturating_sub(1);
            if state.mode == GridMode::Square {
                grid_w = (grid_w / 2) * 2;
            }
            let dash_w_adjusted = main_area.width.saturating_sub(grid_w).saturating_sub(1);
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(grid_w),
                    Constraint::Length(1),
                    Constraint::Length(dash_w_adjusted),
                ])
                .split(main_area)
        } else {
            let mut grid_w = main_area.width;
            if state.mode == GridMode::Square {
                grid_w = (grid_w / 2) * 2;
            }
            let remaining_w = main_area.width.saturating_sub(grid_w);
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(grid_w),
                    Constraint::Length(remaining_w),
                ])
                .split(main_area)
        };
        (chunks, current_dash_w)
    };
    
    let grid_rect = horizontal_chunks[0];
    
    state.check_resize(main_area.width as usize, main_area.height as usize);
    
    let active_style = &STYLES[state.style_idx];
    let (active_cell_color, active_bg_color) = if state.use_custom_colors {
        (
            Color::Rgb(state.theme_custom_r, state.theme_custom_g, state.theme_custom_b),
            Color::Rgb(state.theme_custom_bg_r, state.theme_custom_bg_g, state.theme_custom_bg_b),
        )
    } else {
        (active_style.color, active_style.bg_color)
    };
    let bg_style = Style::default().bg(active_bg_color);
    
    f.render_widget(Block::default().style(bg_style), grid_rect);
    
    if current_dash_w > 0 {
        let border_rect = horizontal_chunks[1];
        let dash_rect = horizontal_chunks[2];
        
        f.render_widget(
            Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(Color::Rgb(88, 91, 112))), // surface
            border_rect
        );
        
        f.render_widget(Block::default().style(bg_style), dash_rect);
        
        let stats = Stats {
            mode: state.mode,
            graph_mode: state.graph_mode,
            generation: state.generation,
            alive: state.alive_count(),
            total: state.w_sim * state.h_sim,
            max_alive: state.max_alive,
            min_alive: state.min_alive,
            density: if state.w_sim * state.h_sim > 0 {
                (state.alive_count() as f64 / (state.w_sim * state.h_sim) as f64) * 100.0
            } else {
                0.0
            },
            growth: state.growth,
            paused: state.paused,
            delay: state.delay,
            birth_rules: state.birth_rules,
            survive_rules: state.survive_rules,
            fps_actual: state.fps_actual,
        };
        
        let dash_lines = render_dashboard(
            dash_rect.width as usize,
            dash_rect.height as usize,
            &stats,
            &state.pop_history,
            active_style,
            active_cell_color,
            active_bg_color,
        );
        
        f.render_widget(Paragraph::new(dash_lines).style(bg_style), dash_rect);
    } else {
        if horizontal_chunks.len() > 1 {
            f.render_widget(Block::default().style(bg_style), horizontal_chunks[1]);
        }
    }
    
    let buf = f.buffer_mut();
    
    if state.mode == GridMode::Square {
        let w_render = grid_rect.width as usize / 2;
        let h_render = grid_rect.height as usize;
        
        for r in 0..h_render {
            let y = grid_rect.y + r as u16;
            let r_offset = r * state.w_sim;
            for c in 0..w_render {
                let x = grid_rect.x + (c * 2) as u16;
                let cell = state.grid[r_offset + c];
                
                let (ch, cell_style) = if active_style.is_solid {
                    let color = if cell { active_cell_color } else { active_bg_color };
                    ("  ", Style::default().bg(color))
                } else {
                    let text = if cell { active_style.alive_str } else { active_style.dead_str };
                    let fg = if cell { active_cell_color } else { Color::Rgb(88, 91, 112) }; // surface for dead
                    (text, Style::default().fg(fg).bg(active_bg_color))
                };
                
                buf.set_string(x, y, ch, cell_style);
            }
        }
    } else {
        let w_render = grid_rect.width as usize;
        let h_render = grid_rect.height as usize;
        
        for r in 0..h_render {
            let y = grid_rect.y + r as u16;
            for c in 0..w_render {
                let x = grid_rect.x + c as u16;
                let ch = get_braille_char(&state.grid, state.w_sim, state.h_sim, c, r);
                let cell_style = Style::default().fg(active_cell_color).bg(active_bg_color);
                buf.set_string(x, y, ch.to_string(), cell_style);
            }
        }
    }
    
    let status_text = make_status_text(
        state.paused,
        state.generation,
        state.alive_count(),
        state.delay,
        state.fps_actual,
        state.show_dashboard,
        status_area.width as usize
    );
    let status_paragraph = Paragraph::new(status_text)
        .style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_widget(status_paragraph, status_area);
    
    // Popups rendern
    match state.popup {
        ActivePopup::None => {}
        ActivePopup::Help => {
            render_help_popup(f, area, state);
        }
        ActivePopup::RulesEditor => {
            render_rules_popup(f, area, state);
        }
        ActivePopup::ThemeManager => {
            render_theme_popup(f, area, state);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| draw_ui(f, &mut state))?;

        let timeout = if state.is_animating() {
            std::time::Duration::from_millis(16) // 60 FPS für Animationen
        } else {
            state.delay.saturating_sub(last_tick.elapsed())
        };

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if state.popup != ActivePopup::None {
                        match state.popup {
                            ActivePopup::None => {}
                            ActivePopup::Help => {
                                match key.code {
                                    KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') | KeyCode::Esc => {
                                        state.popup = ActivePopup::None;
                                    }
                                    _ => {}
                                }
                            }
                            ActivePopup::RulesEditor => {
                                match key.code {
                                    KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Esc => {
                                        state.popup = ActivePopup::None;
                                    }
                                    KeyCode::Tab => {
                                        state.rules_focus = match state.rules_focus {
                                            RulesFocus::PresetList => RulesFocus::TotalisticMatrix,
                                            RulesFocus::TotalisticMatrix => RulesFocus::NeighborhoodMask,
                                            RulesFocus::NeighborhoodMask => RulesFocus::DensitySlider,
                                            RulesFocus::DensitySlider => RulesFocus::PresetList,
                                        };
                                    }
                                    KeyCode::Up => {
                                        match state.rules_focus {
                                            RulesFocus::PresetList => {
                                                state.rules_preset_idx = state.rules_preset_idx.saturating_sub(1);
                                            }
                                            RulesFocus::TotalisticMatrix => {
                                                if state.rules_matrix_row > 0 {
                                                    state.rules_matrix_row -= 1;
                                                }
                                            }
                                            RulesFocus::NeighborhoodMask => {
                                                state.rules_mask_idx = match state.rules_mask_idx {
                                                    3 => 0,
                                                    4 => 2,
                                                    5 => 3,
                                                    6 => 1,
                                                    7 => 4,
                                                    other => other,
                                                };
                                            }
                                            RulesFocus::DensitySlider => {}
                                        }
                                    }
                                    KeyCode::Down => {
                                        match state.rules_focus {
                                            RulesFocus::PresetList => {
                                                if state.rules_preset_idx + 1 < PRESET_RULESETS.len() {
                                                    state.rules_preset_idx += 1;
                                                }
                                            }
                                            RulesFocus::TotalisticMatrix => {
                                                if state.rules_matrix_row < 1 {
                                                    state.rules_matrix_row += 1;
                                                }
                                            }
                                            RulesFocus::NeighborhoodMask => {
                                                state.rules_mask_idx = match state.rules_mask_idx {
                                                    0 => 3,
                                                    1 => 6,
                                                    2 => 4,
                                                    3 => 5,
                                                    4 => 7,
                                                    other => other,
                                                };
                                            }
                                            RulesFocus::DensitySlider => {}
                                        }
                                    }
                                    KeyCode::Left => {
                                        match state.rules_focus {
                                            RulesFocus::PresetList => {}
                                            RulesFocus::TotalisticMatrix => {
                                                state.rules_matrix_col = state.rules_matrix_col.saturating_sub(1);
                                            }
                                            RulesFocus::NeighborhoodMask => {
                                                state.rules_mask_idx = match state.rules_mask_idx {
                                                    1 => 0,
                                                    2 => 1,
                                                    4 => 3,
                                                    6 => 5,
                                                    7 => 6,
                                                    other => other,
                                                };
                                            }
                                            RulesFocus::DensitySlider => {
                                                state.random_density = (state.random_density - 0.01).max(0.05);
                                            }
                                        }
                                    }
                                    KeyCode::Right => {
                                        match state.rules_focus {
                                            RulesFocus::PresetList => {}
                                            RulesFocus::TotalisticMatrix => {
                                                if state.rules_matrix_col < 8 {
                                                    state.rules_matrix_col += 1;
                                                }
                                            }
                                            RulesFocus::NeighborhoodMask => {
                                                state.rules_mask_idx = match state.rules_mask_idx {
                                                    0 => 1,
                                                    1 => 2,
                                                    3 => 4,
                                                    5 => 6,
                                                    6 => 7,
                                                    other => other,
                                                };
                                            }
                                            RulesFocus::DensitySlider => {
                                                state.random_density = (state.random_density + 0.01).min(0.95);
                                            }
                                        }
                                    }
                                    KeyCode::Char(' ') => {
                                        match state.rules_focus {
                                            RulesFocus::PresetList => {
                                                let preset = &PRESET_RULESETS[state.rules_preset_idx];
                                                state.birth_rules = preset.birth;
                                                state.survive_rules = preset.survive;
                                                state.rules_custom_birth = preset.birth;
                                                state.rules_custom_survive = preset.survive;
                                            }
                                            RulesFocus::TotalisticMatrix => {
                                                if state.rules_matrix_row == 0 {
                                                    state.rules_custom_birth[state.rules_matrix_col] = !state.rules_custom_birth[state.rules_matrix_col];
                                                } else {
                                                    state.rules_custom_survive[state.rules_matrix_col] = !state.rules_custom_survive[state.rules_matrix_col];
                                                }
                                                state.birth_rules = state.rules_custom_birth;
                                                state.survive_rules = state.rules_custom_survive;
                                            }
                                            RulesFocus::NeighborhoodMask => {
                                                state.rules_neighbor_mask[state.rules_mask_idx] = !state.rules_neighbor_mask[state.rules_mask_idx];
                                            }
                                            RulesFocus::DensitySlider => {}
                                        }
                                    }
                                    KeyCode::Enter => {
                                        match state.rules_focus {
                                            RulesFocus::PresetList => {
                                                let preset = &PRESET_RULESETS[state.rules_preset_idx];
                                                state.birth_rules = preset.birth;
                                                state.survive_rules = preset.survive;
                                                state.rules_custom_birth = preset.birth;
                                                state.rules_custom_survive = preset.survive;
                                            }
                                            RulesFocus::TotalisticMatrix | RulesFocus::NeighborhoodMask | RulesFocus::DensitySlider => {
                                                state.birth_rules = state.rules_custom_birth;
                                                state.survive_rules = state.rules_custom_survive;
                                            }
                                        }
                                        state.popup = ActivePopup::None;
                                    }
                                    KeyCode::Char('r') | KeyCode::Char('R') => {
                                        state.resize_grid(state.grid_w_last, state.grid_h_last, true);
                                    }
                                    _ => {}
                                }
                            }
                            ActivePopup::ThemeManager => {
                                match key.code {
                                    KeyCode::Char('t') | KeyCode::Char('T') | KeyCode::Esc => {
                                        state.popup = ActivePopup::None;
                                    }
                                    KeyCode::Tab => {
                                        state.theme_focus = match state.theme_focus {
                                            ThemeFocus::PresetList => ThemeFocus::FgRed,
                                            ThemeFocus::FgRed => ThemeFocus::FgGreen,
                                            ThemeFocus::FgGreen => ThemeFocus::FgBlue,
                                            ThemeFocus::FgBlue => ThemeFocus::BgRed,
                                            ThemeFocus::BgRed => ThemeFocus::BgGreen,
                                            ThemeFocus::BgGreen => ThemeFocus::BgBlue,
                                            ThemeFocus::BgBlue => ThemeFocus::CustomCheckbox,
                                            ThemeFocus::CustomCheckbox => ThemeFocus::PresetList,
                                        };
                                    }
                                    KeyCode::Up => {
                                        match state.theme_focus {
                                            ThemeFocus::PresetList => {
                                                state.style_idx = state.style_idx.saturating_sub(1);
                                                if !state.use_custom_colors {
                                                    let active_preset = &STYLES[state.style_idx];
                                                    let (fg_r, fg_g, fg_b) = color_to_rgb(active_preset.color);
                                                    let (bg_r, bg_g, bg_b) = color_to_rgb(active_preset.bg_color);
                                                    state.theme_custom_r = fg_r;
                                                    state.theme_custom_g = fg_g;
                                                    state.theme_custom_b = fg_b;
                                                    state.theme_custom_bg_r = bg_r;
                                                    state.theme_custom_bg_g = bg_g;
                                                    state.theme_custom_bg_b = bg_b;
                                                }
                                            }
                                            ThemeFocus::FgRed => state.theme_focus = ThemeFocus::PresetList,
                                            ThemeFocus::FgGreen => state.theme_focus = ThemeFocus::FgRed,
                                            ThemeFocus::FgBlue => state.theme_focus = ThemeFocus::FgGreen,
                                            ThemeFocus::BgRed => state.theme_focus = ThemeFocus::FgBlue,
                                            ThemeFocus::BgGreen => state.theme_focus = ThemeFocus::BgRed,
                                            ThemeFocus::BgBlue => state.theme_focus = ThemeFocus::BgGreen,
                                            ThemeFocus::CustomCheckbox => state.theme_focus = ThemeFocus::BgBlue,
                                        }
                                    }
                                    KeyCode::Down => {
                                        match state.theme_focus {
                                            ThemeFocus::PresetList => {
                                                if state.style_idx + 1 < STYLES.len() {
                                                    state.style_idx += 1;
                                                    if !state.use_custom_colors {
                                                        let active_preset = &STYLES[state.style_idx];
                                                        let (fg_r, fg_g, fg_b) = color_to_rgb(active_preset.color);
                                                        let (bg_r, bg_g, bg_b) = color_to_rgb(active_preset.bg_color);
                                                        state.theme_custom_r = fg_r;
                                                        state.theme_custom_g = fg_g;
                                                        state.theme_custom_b = fg_b;
                                                        state.theme_custom_bg_r = bg_r;
                                                        state.theme_custom_bg_g = bg_g;
                                                        state.theme_custom_bg_b = bg_b;
                                                    }
                                                }
                                            }
                                            ThemeFocus::FgRed => state.theme_focus = ThemeFocus::FgGreen,
                                            ThemeFocus::FgGreen => state.theme_focus = ThemeFocus::FgBlue,
                                            ThemeFocus::FgBlue => state.theme_focus = ThemeFocus::BgRed,
                                            ThemeFocus::BgRed => state.theme_focus = ThemeFocus::BgGreen,
                                            ThemeFocus::BgGreen => state.theme_focus = ThemeFocus::BgBlue,
                                            ThemeFocus::BgBlue => state.theme_focus = ThemeFocus::CustomCheckbox,
                                            ThemeFocus::CustomCheckbox => state.theme_focus = ThemeFocus::PresetList,
                                        }
                                    }
                                    KeyCode::Left => {
                                        match state.theme_focus {
                                            ThemeFocus::FgRed => { state.theme_custom_r = state.theme_custom_r.saturating_sub(5); state.use_custom_colors = true; }
                                            ThemeFocus::FgGreen => { state.theme_custom_g = state.theme_custom_g.saturating_sub(5); state.use_custom_colors = true; }
                                            ThemeFocus::FgBlue => { state.theme_custom_b = state.theme_custom_b.saturating_sub(5); state.use_custom_colors = true; }
                                            ThemeFocus::BgRed => { state.theme_custom_bg_r = state.theme_custom_bg_r.saturating_sub(5); state.use_custom_colors = true; }
                                            ThemeFocus::BgGreen => { state.theme_custom_bg_g = state.theme_custom_bg_g.saturating_sub(5); state.use_custom_colors = true; }
                                            ThemeFocus::BgBlue => { state.theme_custom_bg_b = state.theme_custom_bg_b.saturating_sub(5); state.use_custom_colors = true; }
                                            _ => {}
                                        }
                                    }
                                    KeyCode::Right => {
                                        match state.theme_focus {
                                            ThemeFocus::FgRed => { state.theme_custom_r = state.theme_custom_r.saturating_add(5); state.use_custom_colors = true; }
                                            ThemeFocus::FgGreen => { state.theme_custom_g = state.theme_custom_g.saturating_add(5); state.use_custom_colors = true; }
                                            ThemeFocus::FgBlue => { state.theme_custom_b = state.theme_custom_b.saturating_add(5); state.use_custom_colors = true; }
                                            ThemeFocus::BgRed => { state.theme_custom_bg_r = state.theme_custom_bg_r.saturating_add(5); state.use_custom_colors = true; }
                                            ThemeFocus::BgGreen => { state.theme_custom_bg_g = state.theme_custom_bg_g.saturating_add(5); state.use_custom_colors = true; }
                                            ThemeFocus::BgBlue => { state.theme_custom_bg_b = state.theme_custom_bg_b.saturating_add(5); state.use_custom_colors = true; }
                                            _ => {}
                                        }
                                    }
                                    KeyCode::Char(' ') => {
                                        match state.theme_focus {
                                            ThemeFocus::CustomCheckbox => {
                                                state.use_custom_colors = !state.use_custom_colors;
                                                if !state.use_custom_colors {
                                                    let active_preset = &STYLES[state.style_idx];
                                                    let (fg_r, fg_g, fg_b) = color_to_rgb(active_preset.color);
                                                    let (bg_r, bg_g, bg_b) = color_to_rgb(active_preset.bg_color);
                                                    state.theme_custom_r = fg_r;
                                                    state.theme_custom_g = fg_g;
                                                    state.theme_custom_b = fg_b;
                                                    state.theme_custom_bg_r = bg_r;
                                                    state.theme_custom_bg_g = bg_g;
                                                    state.theme_custom_bg_b = bg_b;
                                                }
                                            }
                                            ThemeFocus::PresetList => {
                                                state.use_custom_colors = false;
                                                let active_preset = &STYLES[state.style_idx];
                                                let (fg_r, fg_g, fg_b) = color_to_rgb(active_preset.color);
                                                let (bg_r, bg_g, bg_b) = color_to_rgb(active_preset.bg_color);
                                                state.theme_custom_r = fg_r;
                                                state.theme_custom_g = fg_g;
                                                state.theme_custom_b = fg_b;
                                                state.theme_custom_bg_r = bg_r;
                                                state.theme_custom_bg_g = bg_g;
                                                state.theme_custom_bg_b = bg_b;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char(' ') => state.paused = !state.paused,
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => {
                                state.paused = true;
                                state.step_triggered = true;
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                state.show_dashboard = !state.show_dashboard;
                                state.target_dash_width_pct = if state.show_dashboard { 1.0 } else { 0.0 };
                            }
                            KeyCode::Char('m') | KeyCode::Char('M') => {
                                state.change_mode();
                            }
                            KeyCode::Char('v') | KeyCode::Char('V') => {
                                state.graph_mode = match state.graph_mode {
                                    GraphMode::Rolling => GraphMode::Full,
                                    GraphMode::Full => GraphMode::Rolling,
                                };
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                state.resize_grid(state.grid_w_last, state.grid_h_last, true);
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                state.grid = vec![false; state.w_sim * state.h_sim];
                                state.generation = 0;
                                state.max_alive = 0;
                                state.min_alive = 0;
                                state.pop_history = vec![0];
                                state.growth = 0;
                            }
                             KeyCode::Char('s') | KeyCode::Char('S') => {
                                 state.style_idx = (state.style_idx + 1) % STYLES.len();
                                 state.use_custom_colors = false;
                                 let active_preset = &STYLES[state.style_idx];
                                 let (fg_r, fg_g, fg_b) = color_to_rgb(active_preset.color);
                                 let (bg_r, bg_g, bg_b) = color_to_rgb(active_preset.bg_color);
                                 state.theme_custom_r = fg_r;
                                 state.theme_custom_g = fg_g;
                                 state.theme_custom_b = fg_b;
                                 state.theme_custom_bg_r = bg_r;
                                 state.theme_custom_bg_g = bg_g;
                                 state.theme_custom_bg_b = bg_b;
                             }
                            KeyCode::Char('+') => {
                                let step = if state.delay > std::time::Duration::from_millis(10) {
                                    std::time::Duration::from_millis(10)
                                } else {
                                    std::time::Duration::from_millis(1)
                                };
                                state.delay = state.delay.saturating_sub(step);
                            }
                            KeyCode::Char('-') => {
                                let step = if state.delay < std::time::Duration::from_millis(10) {
                                    std::time::Duration::from_millis(1)
                                } else {
                                    std::time::Duration::from_millis(10)
                                };
                                state.delay = state.delay.saturating_add(step);
                                if state.delay > std::time::Duration::from_millis(1000) {
                                    state.delay = std::time::Duration::from_millis(1000);
                                }
                            }
                            KeyCode::Char('e') | KeyCode::Char('E') => {
                                state.popup = ActivePopup::RulesEditor;
                            }
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                state.popup = ActivePopup::ThemeManager;
                            }
                            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => {
                                state.popup = ActivePopup::Help;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= state.delay || state.is_animating() {
            state.tick();
            if last_tick.elapsed() >= state.delay {
                last_tick = std::time::Instant::now();
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        Show
    )?;
    terminal.show_cursor()?;
    Ok(())
}
