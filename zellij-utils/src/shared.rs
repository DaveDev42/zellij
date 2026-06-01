//! Some general utility functions.

use std::net::{IpAddr, Ipv4Addr};
use std::{iter, str::from_utf8};

use crate::data::{Palette, PaletteColor, PaletteSource, ThemeHue};
use crate::envs::get_session_name;
use crate::errors::prelude::*;
use crate::input::options::Options;
use colorsys::{Ansi256, Rgb};
use strip_ansi_escapes::strip;
use unicode_width::UnicodeWidthStr;

#[cfg(unix)]
pub use unix_only::*;

#[cfg(unix)]
mod unix_only {
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::{fs, io};

    pub fn set_permissions(path: &Path, mode: u32) -> io::Result<()> {
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(mode);
        fs::set_permissions(path, permissions)
    }
}

#[cfg(not(unix))]
pub fn set_permissions(_path: &std::path::Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

pub fn ansi_len(s: &str) -> usize {
    from_utf8(&strip(s).unwrap()).unwrap().width()
}

pub fn clean_string_from_control_and_linebreak(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            !c.is_control() &&
            *c != '\n' &&      // line feed
            *c != '\r' &&      // carriage return
            *c != '\u{2028}' && // line separator
            *c != '\u{2029}' // paragraph separator
        })
        .collect()
}

pub fn adjust_to_size(s: &str, rows: usize, columns: usize) -> String {
    s.lines()
        .map(|l| {
            let actual_len = ansi_len(l);
            if actual_len > columns {
                let mut line = String::from(l);
                line.truncate(columns);
                line
            } else {
                [l, &str::repeat(" ", columns - ansi_len(l))].concat()
            }
        })
        .chain(iter::repeat(str::repeat(" ", columns)))
        .take(rows)
        .collect::<Vec<_>>()
        .join("\n\r")
}

pub fn make_terminal_title(pane_title: &str) -> String {
    format!(
        "\u{1b}]0;{}{}\u{07}",
        get_session_name()
            .map(|n| if pane_title.is_empty() {
                format!("{}", n)
            } else {
                format!("{} | ", n)
            })
            .unwrap_or_default(),
        pane_title
    )
}

// Colors
pub mod colors {
    pub const WHITE: u8 = 255;
    pub const GREEN: u8 = 154;
    pub const GRAY: u8 = 238;
    pub const BRIGHT_GRAY: u8 = 245;
    pub const RED: u8 = 124;
    pub const ORANGE: u8 = 166;
    pub const BLACK: u8 = 16;
    pub const MAGENTA: u8 = 201;
    pub const CYAN: u8 = 51;
    pub const YELLOW: u8 = 226;
    pub const BLUE: u8 = 45;
    pub const PURPLE: u8 = 99;
    pub const GOLD: u8 = 136;
    pub const SILVER: u8 = 245;
    pub const PINK: u8 = 207;
    pub const BROWN: u8 = 215;
}

pub fn _hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    Rgb::from_hex_str(hex)
        .expect("The passed argument must be a valid hex color")
        .into()
}

pub fn eightbit_to_rgb(c: u8) -> (u8, u8, u8) {
    Ansi256::new(c).as_rgb().into()
}

/// Convert an 8-bit-per-channel RGB triple to HSB (a.k.a. HSV).
///
/// Returns `(hue, saturation, brightness)` where `hue` is in `[0.0, 360.0)`
/// degrees and `saturation`/`brightness` are in `[0.0, 1.0]`. Achromatic
/// inputs (r == g == b) yield `hue == 0.0`, `saturation == 0.0`.
pub fn rgb_to_hsb(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (hue, saturation, max)
}

/// Convert an HSB (HSV) triple back to 8-bit-per-channel RGB.
///
/// `hue` is taken modulo 360; `saturation` and `brightness` are clamped to
/// `[0.0, 1.0]`. Inverse of [`rgb_to_hsb`] up to rounding.
pub fn hsb_to_rgb(hue: f32, saturation: f32, brightness: f32) -> (u8, u8, u8) {
    let hue = hue.rem_euclid(360.0);
    let saturation = saturation.clamp(0.0, 1.0);
    let brightness = brightness.clamp(0.0, 1.0);

    let c = brightness * saturation;
    let x = c * (1.0 - (((hue / 60.0) % 2.0) - 1.0).abs());
    let m = brightness - c;

    let (r1, g1, b1) = match hue {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let to_u8 = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to_u8(r1), to_u8(g1), to_u8(b1))
}

/// Apply a per-axis HSB multiplier to an RGB color, used to dim inactive
/// panes without claiming any frame space.
///
/// `hsb` is `(hue_mul, saturation_mul, brightness_mul)`: each axis of the
/// color's HSB representation is multiplied by the corresponding factor. This
/// per-axis multiply is analogous to WezTerm's `inactive_pane_hsb`, but zellij
/// encodes each axis as an integer percent at the config layer (100 = identity)
/// whereas WezTerm uses floats (1.0 = identity); here the percents have already
/// been converted to float multipliers. A typical dim is something like
/// `(1.0, 0.85, 0.6)` — hue unchanged, slightly desaturated, noticeably
/// darker. Multipliers `> 1.0` are allowed (saturation/brightness re-clamp on
/// the way back to RGB).
pub fn apply_hsb(rgb: (u8, u8, u8), hsb: (f32, f32, f32)) -> (u8, u8, u8) {
    let (r, g, b) = rgb;
    let (h, s, v) = rgb_to_hsb(r, g, b);
    let (hm, sm, vm) = hsb;
    hsb_to_rgb(h * hm, s * sm, v * vm)
}

pub fn default_palette() -> Palette {
    Palette {
        source: PaletteSource::Default,
        theme_hue: ThemeHue::Dark,
        fg: PaletteColor::EightBit(colors::BRIGHT_GRAY),
        bg: PaletteColor::EightBit(colors::GRAY),
        black: PaletteColor::EightBit(colors::BLACK),
        red: PaletteColor::EightBit(colors::RED),
        green: PaletteColor::EightBit(colors::GREEN),
        yellow: PaletteColor::EightBit(colors::YELLOW),
        blue: PaletteColor::EightBit(colors::BLUE),
        magenta: PaletteColor::EightBit(colors::MAGENTA),
        cyan: PaletteColor::EightBit(colors::CYAN),
        white: PaletteColor::EightBit(colors::WHITE),
        orange: PaletteColor::EightBit(colors::ORANGE),
        gray: PaletteColor::EightBit(colors::GRAY),
        purple: PaletteColor::EightBit(colors::PURPLE),
        gold: PaletteColor::EightBit(colors::GOLD),
        silver: PaletteColor::EightBit(colors::SILVER),
        pink: PaletteColor::EightBit(colors::PINK),
        brown: PaletteColor::EightBit(colors::BROWN),
    }
}

// Dark magic
pub fn detect_theme_hue(bg: PaletteColor) -> ThemeHue {
    match bg {
        PaletteColor::Rgb((r, g, b)) => {
            // HSP, P stands for perceived brightness
            let hsp: f64 = (0.299 * (r as f64 * r as f64)
                + 0.587 * (g as f64 * g as f64)
                + 0.114 * (b as f64 * b as f64))
                .sqrt();
            match hsp > 127.5 {
                true => ThemeHue::Light,
                false => ThemeHue::Dark,
            }
        },
        _ => ThemeHue::Dark,
    }
}

// (this was shamelessly copied from alacritty)
//
// This returns the current terminal version as a unique number based on the
// semver version. The different versions are padded to ensure that a higher semver version will
// always report a higher version number.
pub fn version_number(mut version: &str) -> usize {
    if let Some(separator) = version.rfind('-') {
        version = &version[..separator];
    }

    let mut version_number = 0;

    let semver_versions = version.split('.');
    for (i, semver_version) in semver_versions.rev().enumerate() {
        let semver_number = semver_version.parse::<usize>().unwrap_or(0);
        version_number += usize::pow(100, i as u32) * semver_number;
    }

    version_number
}

pub fn web_server_base_url(
    web_server_ip: IpAddr,
    web_server_port: u16,
    has_certificate: bool,
    enforce_https_for_localhost: bool,
) -> String {
    let is_loopback = match web_server_ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback(),
        IpAddr::V6(ipv6) => ipv6.is_loopback(),
    };

    let url_prefix = if is_loopback && !enforce_https_for_localhost && !has_certificate {
        "http"
    } else {
        "https"
    };
    format!("{}://{}:{}", url_prefix, web_server_ip, web_server_port)
}

pub fn web_server_base_url_from_config(config_options: Options) -> String {
    let web_server_ip = config_options
        .web_server_ip
        .unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let web_server_port = config_options.web_server_port.unwrap_or_else(|| 8082);
    let has_certificate =
        config_options.web_server_cert.is_some() && config_options.web_server_key.is_some();
    let enforce_https_for_localhost = config_options.enforce_https_for_localhost.unwrap_or(false);
    web_server_base_url(
        web_server_ip,
        web_server_port,
        has_certificate,
        enforce_https_for_localhost,
    )
}

pub struct ServerAddress {
    pub ip: String,
    pub port: u16,
}

pub fn parse_base_url(url: &str) -> Result<ServerAddress> {
    let url = url::Url::parse(url)?;
    let ip = url
        .host_str()
        .ok_or_else(|| anyhow!("No host in URL"))?
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or_else(|| anyhow!("No port in URL"))?;

    Ok(ServerAddress { ip, port })
}

#[cfg(test)]
mod hsb_tests {
    use super::{apply_hsb, hsb_to_rgb, rgb_to_hsb};

    fn assert_close(a: (u8, u8, u8), b: (u8, u8, u8)) {
        // Allow ±1 per channel for f32 rounding through HSB and back.
        for (x, y) in [(a.0, b.0), (a.1, b.1), (a.2, b.2)] {
            assert!(
                (x as i16 - y as i16).abs() <= 1,
                "channel mismatch: {:?} vs {:?}",
                a,
                b
            );
        }
    }

    #[test]
    fn rgb_hsb_roundtrip_primaries_and_grays() {
        let samples = [
            (0, 0, 0),
            (255, 255, 255),
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (255, 255, 0),
            (0, 255, 255),
            (255, 0, 255),
            (128, 128, 128),
            (18, 52, 86),
            (200, 30, 90),
        ];
        for rgb in samples {
            let (h, s, b) = rgb_to_hsb(rgb.0, rgb.1, rgb.2);
            assert_close(hsb_to_rgb(h, s, b), rgb);
        }
    }

    #[test]
    fn achromatic_has_zero_hue_and_saturation() {
        let (h, s, _b) = rgb_to_hsb(70, 70, 70);
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn hsb_to_rgb_clamps_out_of_range() {
        // Saturation/brightness above 1.0 and a hue past 360 must not panic
        // and must stay within the 0..=255 channel range.
        let (r, g, b) = hsb_to_rgb(540.0, 2.0, 5.0);
        let _ = (r, g, b); // any u8 is in range by construction
                           // brightness 0 => black regardless of hue/sat
        assert_eq!(hsb_to_rgb(123.0, 0.9, 0.0), (0, 0, 0));
    }

    #[test]
    fn apply_hsb_brightness_multiplier_darkens() {
        // Halving brightness halves each channel of a pure gray.
        assert_close(apply_hsb((200, 200, 200), (1.0, 1.0, 0.5)), (100, 100, 100));
        // Identity multiplier returns the input unchanged.
        assert_close(apply_hsb((10, 120, 240), (1.0, 1.0, 1.0)), (10, 120, 240));
        // Brightness 0 => black for any input.
        assert_close(apply_hsb((10, 120, 240), (1.0, 1.0, 0.0)), (0, 0, 0));
    }

    #[test]
    fn apply_hsb_saturation_multiplier_desaturates_toward_gray() {
        // Fully desaturating a saturated color collapses it onto its
        // brightness gray (max channel preserved).
        let out = apply_hsb((200, 30, 90), (1.0, 0.0, 1.0));
        assert_close(out, (200, 200, 200));
    }
}
