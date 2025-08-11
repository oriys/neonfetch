use crate::animation::styles::hsv_to_rgb;

#[derive(Clone, Copy)]
pub struct Meteor {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub age: f32,
    pub hue: f32,
    pub len: usize,
}

pub fn spawn_meteor(term_w: usize, term_h: usize, speed: f32) -> Meteor {
    let angle_variation = (fastrand::f32() - 0.5) * 0.3; // slight slope differences
    let base_speed = 22.0 * speed.max(0.05);
    let vy = base_speed;
    let vx = base_speed * (1.0 + angle_variation);
    let start_x = fastrand::f32() * (term_w as f32 * 0.4); // spawn a bit left
    let start_y = -(fastrand::f32() * 5.0); // start above view
    let life = (term_h as f32 + term_w as f32) / base_speed + 1.0;
    let hue = fastrand::f32() * 360.0;
    let len = 4 + fastrand::usize(..6); // 4..9
    Meteor { x: start_x, y: start_y, vx, vy, life, age: 0.0, hue, len }
}

pub fn update_meteors(meteors: &mut Vec<Meteor>, dt: f32, term_w: usize, term_h: usize, speed: f32) {
    for m in meteors.iter_mut() {
        m.age += dt;
        m.x += m.vx * dt;
        m.y += m.vy * dt;
    }
    // remove old
    meteors.retain(|m| m.age < m.life && m.x < term_w as f32 + 10.0 && m.y < term_h as f32 + 10.0);
    // spawn logic: maintain target count based on area
    let target = ((term_w * term_h) as f32 / 1200.0).clamp(2.0, 14.0) as usize;
    if meteors.len() < target {
        let spawn_prob = 0.25 * (1.0 + speed).min(2.0);
        if fastrand::f32() < spawn_prob { meteors.push(spawn_meteor(term_w, term_h, speed)); }
    }
}

pub fn sample_cell_color(time: f32, row: usize, col: usize, meteors: &Vec<Meteor>) -> Option<(u8,u8,u8)> {
    // Accumulate contributions (though usually one meteor)
    let mut r_acc=0.0; let mut g_acc=0.0; let mut b_acc=0.0; let mut w_acc=0.0;
    for m in meteors.iter() {
        // head position
        let hx = m.x;
        let hy = m.y;
        // param along meteor tail for this cell: project cell onto direction
        let dx = m.vx;
        let dy = m.vy;
        let dir_len = (dx*dx + dy*dy).sqrt();
        if dir_len == 0.0 { continue; }
        let ux = dx / dir_len;
        let uy = dy / dir_len;
        // vector from head to cell center
        let cx = col as f32 + 0.3 - hx;
        let cy = row as f32 + 0.3 - hy;
        let proj = cx * ux + cy * uy; // distance behind head along path
        if proj < 0.0 || proj > m.len as f32 { continue; }
        // perpendicular distance
        let px = cx - ux * proj;
        let py = cy - uy * proj;
        let perp = (px*px + py*py).sqrt();
        if perp > 1.2 { continue; }
        // brightness envelope along tail (fade toward tail)
        let tail_t = proj / m.len as f32; // 0 head -> 1 tail
        let core = (1.0 - tail_t).powf(1.1);
        let falloff = (-perp * 1.6).exp();
        let mut w = core * falloff;
        // global flicker
        let flicker = (time * 30.0 + m.hue).sin() * 0.15 + 0.85;
        w *= flicker;
        if w < 0.02 { continue; }
        let hue = (m.hue + tail_t * 40.0) % 360.0;
        let sat = 0.65 + (1.0 - tail_t) * 0.25;
        let val = (0.25 + w * 0.9).min(1.0);
        let (r,g,b) = hsv_to_rgb(hue, sat.min(0.95), val);
        r_acc += r as f32 * w;
        g_acc += g as f32 * w;
        b_acc += b as f32 * w;
        w_acc += w;
    }
    if w_acc > 0.0 {
        let inv = 1.0 / w_acc;
        return Some(((r_acc*inv) as u8, (g_acc*inv) as u8, (b_acc*inv) as u8));
    }
    None
}
