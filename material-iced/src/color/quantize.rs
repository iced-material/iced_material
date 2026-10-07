// SPDX-License-Identifier: LGPL-3.0-only

use std::collections::HashMap;

use super::utils;

pub fn celebi(pixels: &[u32], max_colors: usize) -> Vec<(u32, u32)> {
    let wu_clusters = wu(pixels, max_colors);
    wsmeans(pixels, &wu_clusters, max_colors)
}

const INDEX_BITS: i32 = 5;
const INDEX_COUNT: i32 = 33;
const TOTAL_SIZE: usize = 35937;

#[derive(Clone, Copy, Default)]
struct Cube {
    r0: i32,
    r1: i32,
    g0: i32,
    g1: i32,
    b0: i32,
    b1: i32,
    vol: i32,
}

#[derive(Clone, Copy)]
enum Direction {
    Red,
    Green,
    Blue,
}

struct Moments {
    weights: Vec<i32>,
    r: Vec<i32>,
    g: Vec<i32>,
    b: Vec<i32>,
    m2: Vec<f64>,
}

fn index(r: i32, g: i32, b: i32) -> usize {
    ((r << (INDEX_BITS * 2)) + (r << (INDEX_BITS + 1)) + r + (g << INDEX_BITS) + g + b) as usize
}

fn volume(cube: &Cube, moment: &[i32]) -> i32 {
    moment[index(cube.r1, cube.g1, cube.b1)]
        .wrapping_sub(moment[index(cube.r1, cube.g1, cube.b0)])
        .wrapping_sub(moment[index(cube.r1, cube.g0, cube.b1)])
        .wrapping_add(moment[index(cube.r1, cube.g0, cube.b0)])
        .wrapping_sub(moment[index(cube.r0, cube.g1, cube.b1)])
        .wrapping_add(moment[index(cube.r0, cube.g1, cube.b0)])
        .wrapping_add(moment[index(cube.r0, cube.g0, cube.b1)])
        .wrapping_sub(moment[index(cube.r0, cube.g0, cube.b0)])
}

fn bottom(cube: &Cube, direction: Direction, moment: &[i32]) -> i32 {
    let (a, b, c, d) = match direction {
        Direction::Red => (
            index(cube.r0, cube.g1, cube.b1),
            index(cube.r0, cube.g1, cube.b0),
            index(cube.r0, cube.g0, cube.b1),
            index(cube.r0, cube.g0, cube.b0),
        ),
        Direction::Green => (
            index(cube.r1, cube.g0, cube.b1),
            index(cube.r1, cube.g0, cube.b0),
            index(cube.r0, cube.g0, cube.b1),
            index(cube.r0, cube.g0, cube.b0),
        ),
        Direction::Blue => (
            index(cube.r1, cube.g1, cube.b0),
            index(cube.r1, cube.g0, cube.b0),
            index(cube.r0, cube.g1, cube.b0),
            index(cube.r0, cube.g0, cube.b0),
        ),
    };
    moment[a]
        .wrapping_neg()
        .wrapping_add(moment[b])
        .wrapping_add(moment[c])
        .wrapping_sub(moment[d])
}

fn top(cube: &Cube, direction: Direction, position: i32, moment: &[i32]) -> i32 {
    let (a, b, c, d) = match direction {
        Direction::Red => (
            index(position, cube.g1, cube.b1),
            index(position, cube.g1, cube.b0),
            index(position, cube.g0, cube.b1),
            index(position, cube.g0, cube.b0),
        ),
        Direction::Green => (
            index(cube.r1, position, cube.b1),
            index(cube.r1, position, cube.b0),
            index(cube.r0, position, cube.b1),
            index(cube.r0, position, cube.b0),
        ),
        Direction::Blue => (
            index(cube.r1, cube.g1, position),
            index(cube.r1, cube.g0, position),
            index(cube.r0, cube.g1, position),
            index(cube.r0, cube.g0, position),
        ),
    };
    moment[a]
        .wrapping_sub(moment[b])
        .wrapping_sub(moment[c])
        .wrapping_add(moment[d])
}

fn squared_sum(r: i32, g: i32, b: i32) -> f64 {
    r.wrapping_mul(r)
        .wrapping_add(g.wrapping_mul(g))
        .wrapping_add(b.wrapping_mul(b)) as f64
}

fn variance(m: &Moments, cube: &Cube) -> f64 {
    let dr = volume(cube, &m.r);
    let dg = volume(cube, &m.g);
    let db = volume(cube, &m.b);
    let xx = m.m2[index(cube.r1, cube.g1, cube.b1)]
        - m.m2[index(cube.r1, cube.g1, cube.b0)]
        - m.m2[index(cube.r1, cube.g0, cube.b1)]
        + m.m2[index(cube.r1, cube.g0, cube.b0)]
        - m.m2[index(cube.r0, cube.g1, cube.b1)]
        + m.m2[index(cube.r0, cube.g1, cube.b0)]
        + m.m2[index(cube.r0, cube.g0, cube.b1)]
        - m.m2[index(cube.r0, cube.g0, cube.b0)];
    let hypotenuse = dr
        .wrapping_mul(dr)
        .wrapping_add(dg.wrapping_mul(dg))
        .wrapping_add(db.wrapping_mul(db));
    xx - hypotenuse as f64 / volume(cube, &m.weights) as f64
}

#[allow(clippy::too_many_arguments)]
fn maximize(
    m: &Moments,
    cube: &Cube,
    direction: Direction,
    first: i32,
    last: i32,
    whole_r: i32,
    whole_g: i32,
    whole_b: i32,
    whole_w: i32,
) -> (i32, f64) {
    let bottom_r = bottom(cube, direction, &m.r);
    let bottom_g = bottom(cube, direction, &m.g);
    let bottom_b = bottom(cube, direction, &m.b);
    let bottom_w = bottom(cube, direction, &m.weights);
    let mut max = 0.0;
    let mut cut = -1;
    for i in first..last {
        let half_r = bottom_r.wrapping_add(top(cube, direction, i, &m.r));
        let half_g = bottom_g.wrapping_add(top(cube, direction, i, &m.g));
        let half_b = bottom_b.wrapping_add(top(cube, direction, i, &m.b));
        let half_w = bottom_w.wrapping_add(top(cube, direction, i, &m.weights));
        if half_w == 0 {
            continue;
        }
        let mut temp = squared_sum(half_r, half_g, half_b) / half_w as f64;
        let half_r = whole_r.wrapping_sub(half_r);
        let half_g = whole_g.wrapping_sub(half_g);
        let half_b = whole_b.wrapping_sub(half_b);
        let half_w = whole_w.wrapping_sub(half_w);
        if half_w == 0 {
            continue;
        }
        temp += squared_sum(half_r, half_g, half_b) / half_w as f64;
        if temp > max {
            max = temp;
            cut = i;
        }
    }
    (cut, max)
}

fn cut(m: &Moments, one: &mut Cube, two: &mut Cube) -> bool {
    let whole_r = volume(one, &m.r);
    let whole_g = volume(one, &m.g);
    let whole_b = volume(one, &m.b);
    let whole_w = volume(one, &m.weights);
    let (cut_r, max_r) = maximize(
        m,
        one,
        Direction::Red,
        one.r0 + 1,
        one.r1,
        whole_r,
        whole_g,
        whole_b,
        whole_w,
    );
    let (cut_g, max_g) = maximize(
        m,
        one,
        Direction::Green,
        one.g0 + 1,
        one.g1,
        whole_r,
        whole_g,
        whole_b,
        whole_w,
    );
    let (cut_b, max_b) = maximize(
        m,
        one,
        Direction::Blue,
        one.b0 + 1,
        one.b1,
        whole_r,
        whole_g,
        whole_b,
        whole_w,
    );
    let direction = if max_r >= max_g && max_r >= max_b {
        if cut_r < 0 {
            return false;
        }
        Direction::Red
    } else if max_g >= max_r && max_g >= max_b {
        Direction::Green
    } else {
        Direction::Blue
    };
    two.r1 = one.r1;
    two.g1 = one.g1;
    two.b1 = one.b1;
    match direction {
        Direction::Red => {
            one.r1 = cut_r;
            two.r0 = one.r1;
            two.g0 = one.g0;
            two.b0 = one.b0;
        }
        Direction::Green => {
            one.g1 = cut_g;
            two.r0 = one.r0;
            two.g0 = one.g1;
            two.b0 = one.b0;
        }
        Direction::Blue => {
            one.b1 = cut_b;
            two.r0 = one.r0;
            two.g0 = one.g0;
            two.b0 = one.b1;
        }
    }
    one.vol = (one.r1 - one.r0) * (one.g1 - one.g0) * (one.b1 - one.b0);
    two.vol = (two.r1 - two.r0) * (two.g1 - two.g0) * (two.b1 - two.b0);
    true
}

fn wu(pixels: &[u32], max_colors: usize) -> Vec<u32> {
    let mut m = Moments {
        weights: vec![0; TOTAL_SIZE],
        r: vec![0; TOTAL_SIZE],
        g: vec![0; TOTAL_SIZE],
        b: vec![0; TOTAL_SIZE],
        m2: vec![0.0; TOTAL_SIZE],
    };
    let mut count_by_pixel: HashMap<u32, i32> = HashMap::new();
    for &pixel in pixels {
        *count_by_pixel.entry(pixel).or_insert(0) += 1;
    }
    for (&pixel, &count) in &count_by_pixel {
        let red = utils::red_from_argb(pixel) as i32;
        let green = utils::green_from_argb(pixel) as i32;
        let blue = utils::blue_from_argb(pixel) as i32;
        let bits_to_remove = 8 - INDEX_BITS;
        let i = index(
            (red >> bits_to_remove) + 1,
            (green >> bits_to_remove) + 1,
            (blue >> bits_to_remove) + 1,
        );
        m.weights[i] = m.weights[i].wrapping_add(count);
        m.r[i] = m.r[i].wrapping_add(red.wrapping_mul(count));
        m.g[i] = m.g[i].wrapping_add(green.wrapping_mul(count));
        m.b[i] = m.b[i].wrapping_add(blue.wrapping_mul(count));
        m.m2[i] += count.wrapping_mul(red * red + green * green + blue * blue) as f64;
    }
    for r in 1..INDEX_COUNT {
        let mut area = [0i32; INDEX_COUNT as usize];
        let mut area_r = [0i32; INDEX_COUNT as usize];
        let mut area_g = [0i32; INDEX_COUNT as usize];
        let mut area_b = [0i32; INDEX_COUNT as usize];
        let mut area2 = [0f64; INDEX_COUNT as usize];
        for g in 1..INDEX_COUNT {
            let mut line = 0i32;
            let mut line_r = 0i32;
            let mut line_g = 0i32;
            let mut line_b = 0i32;
            let mut line2 = 0.0;
            for b in 1..INDEX_COUNT {
                let i = index(r, g, b);
                let bu = b as usize;
                line = line.wrapping_add(m.weights[i]);
                line_r = line_r.wrapping_add(m.r[i]);
                line_g = line_g.wrapping_add(m.g[i]);
                line_b = line_b.wrapping_add(m.b[i]);
                line2 += m.m2[i];
                area[bu] = area[bu].wrapping_add(line);
                area_r[bu] = area_r[bu].wrapping_add(line_r);
                area_g[bu] = area_g[bu].wrapping_add(line_g);
                area_b[bu] = area_b[bu].wrapping_add(line_b);
                area2[bu] += line2;
                let p = index(r - 1, g, b);
                m.weights[i] = m.weights[p].wrapping_add(area[bu]);
                m.r[i] = m.r[p].wrapping_add(area_r[bu]);
                m.g[i] = m.g[p].wrapping_add(area_g[bu]);
                m.b[i] = m.b[p].wrapping_add(area_b[bu]);
                m.m2[i] = m.m2[p] + area2[bu];
            }
        }
    }

    let mut cubes = vec![Cube::default(); max_colors];
    let mut volume_variance = vec![0.0; max_colors];
    cubes[0].r1 = INDEX_COUNT - 1;
    cubes[0].g1 = INDEX_COUNT - 1;
    cubes[0].b1 = INDEX_COUNT - 1;
    let mut generated = max_colors;
    let mut next = 0usize;
    let mut i = 1usize;
    while i < max_colors {
        let mut one = cubes[next];
        let mut two = cubes[i];
        if cut(&m, &mut one, &mut two) {
            cubes[next] = one;
            cubes[i] = two;
            volume_variance[next] = if cubes[next].vol > 1 {
                variance(&m, &cubes[next])
            } else {
                0.0
            };
            volume_variance[i] = if cubes[i].vol > 1 {
                variance(&m, &cubes[i])
            } else {
                0.0
            };
        } else {
            cubes[next] = one;
            volume_variance[next] = 0.0;
            i -= 1;
        }
        next = 0;
        let mut temp = volume_variance[0];
        for (j, &v) in volume_variance.iter().enumerate().take(i + 1).skip(1) {
            if v > temp {
                temp = v;
                next = j;
            }
        }
        if temp <= 0.0 {
            generated = i + 1;
            break;
        }
        i += 1;
    }

    let mut colors = Vec::new();
    for cube in cubes.iter().take(generated) {
        let weight = volume(cube, &m.weights);
        if weight > 0 {
            let r = volume(cube, &m.r) / weight;
            let g = volume(cube, &m.g) / weight;
            let b = volume(cube, &m.b) / weight;
            let color = 0xFF00_0000
                | ((r & 0xff) as u32) << 16
                | ((g & 0xff) as u32) << 8
                | (b & 0xff) as u32;
            if !colors.contains(&color) {
                colors.push(color);
            }
        }
    }
    colors
}

struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    fn new(seed: i64) -> JavaRandom {
        JavaRandom {
            seed: (seed ^ 0x5DEECE66D) & ((1 << 48) - 1),
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1 << 48) - 1);
        (self.seed as u64 >> (48 - bits)) as i32
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        let mut r = self.next(31);
        let m = bound - 1;
        if bound & m == 0 {
            ((bound as i64 * r as i64) >> 31) as i32
        } else {
            let mut u = r;
            loop {
                r = u % bound;
                if u.wrapping_sub(r).wrapping_add(m) >= 0 {
                    break;
                }
                u = self.next(31);
            }
            r
        }
    }
}

fn lab_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dl = a[0] - b[0];
    let da = a[1] - b[1];
    let db = a[2] - b[2];
    dl * dl + da * da + db * db
}

fn wsmeans(input_pixels: &[u32], starting_clusters: &[u32], max_colors: usize) -> Vec<(u32, u32)> {
    const MAX_ITERATIONS: usize = 10;
    const MIN_MOVEMENT_DISTANCE: f64 = 3.0;
    let mut random = JavaRandom::new(0x42688);
    let mut slot_by_pixel: HashMap<u32, usize> = HashMap::new();
    let mut points: Vec<[f64; 3]> = Vec::new();
    let mut counts: Vec<i32> = Vec::new();
    for &pixel in input_pixels {
        match slot_by_pixel.get(&pixel) {
            Some(&slot) => counts[slot] += 1,
            None => {
                slot_by_pixel.insert(pixel, points.len());
                points.push(utils::lab_from_argb(pixel));
                counts.push(1);
            }
        }
    }
    let point_count = points.len();
    let mut cluster_count = max_colors.min(point_count);
    if !starting_clusters.is_empty() {
        cluster_count = cluster_count.min(starting_clusters.len());
    }
    let mut clusters: Vec<[f64; 3]> = starting_clusters
        .iter()
        .take(cluster_count)
        .map(|&argb| utils::lab_from_argb(argb))
        .collect();
    let mut cluster_indices: Vec<usize> = (0..point_count)
        .map(|_| random.next_int(cluster_count as i32) as usize)
        .collect();
    let mut matrix: Vec<Vec<(f64, i32)>> = vec![vec![(-1.0, -1); cluster_count]; cluster_count];
    let mut pixel_count_sums = vec![0i32; cluster_count];
    for iteration in 0..MAX_ITERATIONS {
        for i in 0..cluster_count {
            for j in (i + 1)..cluster_count {
                let distance = lab_distance(clusters[i], clusters[j]);
                matrix[j][i] = (distance, i as i32);
                matrix[i][j] = (distance, j as i32);
            }
            matrix[i].sort_by(|a, b| a.0.total_cmp(&b.0));
        }
        let mut points_moved = 0;
        for i in 0..point_count {
            let point = points[i];
            let previous_cluster_index = cluster_indices[i];
            let previous_distance = lab_distance(point, clusters[previous_cluster_index]);
            let mut minimum_distance = previous_distance;
            let mut new_cluster_index: Option<usize> = None;
            for j in 0..cluster_count {
                if matrix[previous_cluster_index][j].0 >= 4.0 * previous_distance {
                    continue;
                }
                let distance = lab_distance(point, clusters[j]);
                if distance < minimum_distance {
                    minimum_distance = distance;
                    new_cluster_index = Some(j);
                }
            }
            if let Some(new_index) = new_cluster_index {
                let change = (minimum_distance.sqrt() - previous_distance.sqrt()).abs();
                if change > MIN_MOVEMENT_DISTANCE {
                    points_moved += 1;
                    cluster_indices[i] = new_index;
                }
            }
        }
        if points_moved == 0 && iteration != 0 {
            break;
        }
        let mut sums = vec![[0.0f64; 3]; cluster_count];
        pixel_count_sums.iter_mut().for_each(|s| *s = 0);
        for i in 0..point_count {
            let c = cluster_indices[i];
            let count = counts[i];
            pixel_count_sums[c] += count;
            sums[c][0] += points[i][0] * count as f64;
            sums[c][1] += points[i][1] * count as f64;
            sums[c][2] += points[i][2] * count as f64;
        }
        for i in 0..cluster_count {
            let count = pixel_count_sums[i];
            if count == 0 {
                clusters[i] = [0.0; 3];
                continue;
            }
            let count = count as f64;
            clusters[i] = [sums[i][0] / count, sums[i][1] / count, sums[i][2] / count];
        }
    }
    let mut result: Vec<(u32, u32)> = Vec::new();
    for i in 0..cluster_count {
        let count = pixel_count_sums[i];
        if count == 0 {
            continue;
        }
        let argb = utils::argb_from_lab(clusters[i][0], clusters[i][1], clusters[i][2]);
        if result.iter().any(|(c, _)| *c == argb) {
            continue;
        }
        result.push((argb, count as u32));
    }
    result
}
