// SPDX-License-Identifier: LGPL-3.0-only

use std::f64::consts::PI;
use std::sync::LazyLock;

use super::utils::{self, signum};

pub struct ViewingConditions {
    pub n: f64,
    pub aw: f64,
    pub nbb: f64,
    pub ncb: f64,
    pub c: f64,
    pub nc: f64,
    pub rgb_d: [f64; 3],
    pub fl: f64,
    pub z: f64,
}

pub static DEFAULT_VIEWING_CONDITIONS: LazyLock<ViewingConditions> =
    LazyLock::new(ViewingConditions::standard);

impl ViewingConditions {
    fn standard() -> Self {
        let white_point = utils::WHITE_POINT_D65;
        let adapting_luminance = (200.0 / PI) * utils::y_from_lstar(50.0) / 100.0;
        let background_lstar = 50.0;
        let surround = 2.0;
        let xyz = white_point;
        let r_w = xyz[0] * 0.401288 + xyz[1] * 0.650173 + xyz[2] * -0.051461;
        let g_w = xyz[0] * -0.250268 + xyz[1] * 1.204414 + xyz[2] * 0.045854;
        let b_w = xyz[0] * -0.002079 + xyz[1] * 0.048952 + xyz[2] * 0.953127;
        let f = 0.8 + surround / 10.0;
        let c = if f >= 0.9 {
            utils::lerp(0.59, 0.69, (f - 0.9) * 10.0)
        } else {
            utils::lerp(0.525, 0.59, (f - 0.8) * 10.0)
        };
        let d = f * (1.0 - (1.0 / 3.6) * ((-adapting_luminance - 42.0) / 92.0).exp());
        let d = d.clamp(0.0, 1.0);
        let nc = f;
        let rgb_d = [
            d * (100.0 / r_w) + 1.0 - d,
            d * (100.0 / g_w) + 1.0 - d,
            d * (100.0 / b_w) + 1.0 - d,
        ];
        let k = 1.0 / (5.0 * adapting_luminance + 1.0);
        let k4 = k * k * k * k;
        let k4_f = 1.0 - k4;
        let fl = k4 * adapting_luminance + 0.1 * k4_f * k4_f * (5.0 * adapting_luminance).cbrt();
        let n = utils::y_from_lstar(background_lstar) / white_point[1];
        let z = 1.48 + n.sqrt();
        let nbb = 0.725 / n.powf(0.2);
        let ncb = nbb;
        let factors = [
            (fl * rgb_d[0] * r_w / 100.0).powf(0.42),
            (fl * rgb_d[1] * g_w / 100.0).powf(0.42),
            (fl * rgb_d[2] * b_w / 100.0).powf(0.42),
        ];
        let rgb_a = [
            400.0 * factors[0] / (factors[0] + 27.13),
            400.0 * factors[1] / (factors[1] + 27.13),
            400.0 * factors[2] / (factors[2] + 27.13),
        ];
        let aw = (2.0 * rgb_a[0] + rgb_a[1] + 0.05 * rgb_a[2]) * nbb;
        Self {
            n,
            aw,
            nbb,
            ncb,
            c,
            nc,
            rgb_d,
            fl,
            z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cam16 {
    pub hue: f64,
    pub chroma: f64,
}

impl Cam16 {
    pub fn from_int(argb: u32) -> Cam16 {
        let vc = &*DEFAULT_VIEWING_CONDITIONS;
        let red_l = utils::linearized((argb & 0x00ff_0000) >> 16);
        let green_l = utils::linearized((argb & 0x0000_ff00) >> 8);
        let blue_l = utils::linearized(argb & 0x0000_00ff);
        let x = 0.41233895 * red_l + 0.35762064 * green_l + 0.18051042 * blue_l;
        let y = 0.2126 * red_l + 0.7152 * green_l + 0.0722 * blue_l;
        let z = 0.01932141 * red_l + 0.11916382 * green_l + 0.95034478 * blue_l;
        let r_c = 0.401288 * x + 0.650173 * y - 0.051461 * z;
        let g_c = -0.250268 * x + 1.204414 * y + 0.045854 * z;
        let b_c = -0.002079 * x + 0.048952 * y + 0.953127 * z;
        let r_d = vc.rgb_d[0] * r_c;
        let g_d = vc.rgb_d[1] * g_c;
        let b_d = vc.rgb_d[2] * b_c;
        let r_af = (vc.fl * r_d.abs() / 100.0).powf(0.42);
        let g_af = (vc.fl * g_d.abs() / 100.0).powf(0.42);
        let b_af = (vc.fl * b_d.abs() / 100.0).powf(0.42);
        let r_a = signum(r_d) * 400.0 * r_af / (r_af + 27.13);
        let g_a = signum(g_d) * 400.0 * g_af / (g_af + 27.13);
        let b_a = signum(b_d) * 400.0 * b_af / (b_af + 27.13);
        let a = (11.0 * r_a + -12.0 * g_a + b_a) / 11.0;
        let b = (r_a + g_a - 2.0 * b_a) / 9.0;
        let u = (20.0 * r_a + 20.0 * g_a + 21.0 * b_a) / 20.0;
        let p2 = (40.0 * r_a + 20.0 * g_a + b_a) / 20.0;
        let atan_degrees = b.atan2(a) * 180.0 / PI;
        let hue = utils::sanitize_degrees_double(atan_degrees);
        let ac = p2 * vc.nbb;
        let j = 100.0 * (ac / vc.aw).powf(vc.c * vc.z);
        let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
        let e_hue = 0.25 * ((hue_prime * PI / 180.0 + 2.0).cos() + 3.8);
        let p1 = (50000.0 / 13.0) * e_hue * vc.nc * vc.ncb;
        let t = p1 * (a * a + b * b).sqrt() / (u + 0.305);
        let alpha = t.powf(0.9) * (1.64 - 0.29f64.powf(vc.n)).powf(0.73);
        Cam16 {
            hue,
            chroma: alpha * (j / 100.0).sqrt(),
        }
    }
}
