#![forbid(unsafe_code)]
//! Grace vs Trust Rebuild — two opposing forces, not one parameter.
//!
//! GRACE = willingness to pause and reconsider (enters spindle, COSTLY)
//! TRUST_REBUILD = willingness to cooperate after conflict (exits trap, BENEFICIAL)
//!
//! High grace + low trust rebuild = everyone enters spindle, nobody leaves = death
//! Low grace + high trust rebuild = agents hold ground but reconnect fast = alive
//! Moderate both = adaptive system

/// Parameters for the grace/trust model.
#[derive(Debug, Clone, Copy)]
pub struct GraceParams {
    pub grace_rate: f64,        // 0-1: how often disagreement → enter spindle (keep LOW)
    pub trust_rebuild: f64,     // 0-1: how fast trust recovers after conflict (keep MODERATE)
    pub tunnel_rate: f64,       // 0-1: base tunneling out of spindle
    pub trap_rate: f64,         // 0-1: base trapping into spindle
    pub population: usize,
    pub ticks: usize,
}

impl GraceParams {
    pub fn new(pop: usize, ticks: usize) -> Self {
        Self {
            grace_rate: 0.05,     // Low grace — don't drop to 0 easily
            trust_rebuild: 0.5,   // Moderate trust — do reconnect after conflict
            tunnel_rate: 0.006,
            trap_rate: 0.01,
            population: pop,
            ticks,
        }
    }

    pub fn with_grace(mut self, g: f64) -> Self { self.grace_rate = g; self }
    pub fn with_trust(mut self, t: f64) -> Self { self.trust_rebuild = t; self }
    pub fn with_tunnel(mut self, t: f64) -> Self { self.tunnel_rate = t; self }
    pub fn with_trap(mut self, t: f64) -> Self { self.trap_rate = t; self }
}

/// Result of a grace experiment.
#[derive(Debug, Clone)]
pub struct GraceResult {
    pub params: GraceParams,
    pub survival_rate: f64,
    pub entropy: f64,
    pub mean_abs_gamma: f64,
    pub peak_survival: f64,
    pub collapse_tick: Option<usize>,
    pub trust_events: usize,    // How many trust rebuilds happened
    pub grace_events: usize,    // How many grace pauses happened
    pub tunnel_events: usize,   // How many tunnel-outs happened
    pub trap_events: usize,     // How many trap-ins happened
}

/// Simple LCG.
pub struct Rng { s: u64 }
impl Rng {
    pub fn new(seed: u64) -> Self { Self { s: seed } }
    pub fn next(&mut self) -> f64 {
        self.s = self.s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.s >> 33) as f64 / (1u64 << 31) as f64
    }
}

/// Agent with trust level.
#[derive(Debug, Clone, Copy)]
struct Agent {
    state: i8,
    trust: u8,  // 0-255, starts at 128
}

/// Run a grace experiment.
pub fn run(params: GraceParams, seed: u64) -> GraceResult {
    let mut rng = Rng::new(seed);
    let n = params.population;

    // Initialize agents
    let third = n / 3;
    let mut agents: Vec<Agent> = (0..n).map(|i| Agent {
        state: if i < third { -1 } else if i < 2 * third { 0 } else { 1 },
        trust: 128,
    }).collect();

    // Shuffle
    for i in (1..agents.len()).rev() {
        let j = (rng.next() * i as f64) as usize;
        agents.swap(i, j);
    }

    let mut peak_survival = 0.0f64;
    let mut collapse_tick: Option<usize> = None;
    let mut trust_events = 0usize;
    let mut grace_events = 0usize;
    let mut tunnel_events = 0usize;
    let mut trap_events = 0usize;

    for tick in 0..params.ticks {
        let mut next = agents.clone();

        for i in 0..n {
            let r = rng.next();

            match agents[i].state {
                0 => {
                    // In spindle
                    // Tunnel out: base rate + trust_rebuild bonus
                    let effective_tunnel = params.tunnel_rate + (agents[i].trust as f64 / 255.0) * params.trust_rebuild * 0.01;
                    if r < effective_tunnel {
                        next[i].state = if rng.next() < 0.5 { 1 } else { -1 };
                        next[i].trust = next[i].trust.saturating_add(10);
                        tunnel_events += 1;
                    }
                }
                s @ (-1) | s @ 1 => {
                    // Active
                    // Base trapping
                    if r < params.trap_rate {
                        next[i].state = 0;
                        trap_events += 1;
                        continue;
                    }

                    // Interaction with random neighbor
                    let j = (rng.next() * n as f64) as usize;
                    if agents[j].state != 0 && agents[j].state != s {
                        // Disagreement!
                        // Grace: enter spindle to reconsider
                        if rng.next() < params.grace_rate {
                            next[i].state = 0;
                            next[i].trust = next[i].trust.saturating_sub(20);
                            grace_events += 1;
                        } else {
                            // Trust rebuild: even though we disagree, increment trust slightly
                            next[i].trust = next[i].trust.saturating_add(2);
                            trust_events += 1;
                        }
                    } else if agents[j].state == s {
                        // Agreement — build trust
                        next[i].trust = next[i].trust.saturating_add(5);
                        trust_events += 1;
                    }
                }
                _ => {}
            }
        }

        agents = next;

        // Track metrics
        let active = agents.iter().filter(|a| a.state != 0).count() as f64 / n as f64;
        if active > peak_survival { peak_survival = active; }
        if collapse_tick.is_none() && active < 0.1 {
            collapse_tick = Some(tick);
        }
    }

    // Final metrics
    let active_count = agents.iter().filter(|a| a.state != 0).count();
    let survival = active_count as f64 / n as f64;
    let abs_gamma = agents.iter().map(|a| a.state.abs() as f64).sum::<f64>() / n as f64;

    let n_frac = agents.iter().filter(|a| a.state == -1).count() as f64 / n as f64;
    let z_frac = agents.iter().filter(|a| a.state == 0).count() as f64 / n as f64;
    let p_frac = agents.iter().filter(|a| a.state == 1).count() as f64 / n as f64;
    let mut entropy = 0.0;
    if n_frac > 0.0 { entropy -= n_frac * n_frac.log2(); }
    if z_frac > 0.0 { entropy -= z_frac * z_frac.log2(); }
    if p_frac > 0.0 { entropy -= p_frac * p_frac.log2(); }

    GraceResult {
        params, survival_rate: survival, entropy, mean_abs_gamma: abs_gamma,
        peak_survival, collapse_tick, trust_events, grace_events, tunnel_events, trap_events,
    }
}

/// Sweep grace rate (keeping trust_rebuild fixed).
pub fn sweep_grace(pop: usize, ticks: usize, steps: usize, trust: f64) -> Vec<(f64, GraceResult)> {
    (0..steps).map(|i| {
        let g = i as f64 / (steps - 1).max(1) as f64;
        let p = GraceParams::new(pop, ticks).with_grace(g).with_trust(trust);
        (g, run(p, 42 + i as u64))
    }).collect()
}

/// Sweep trust rebuild (keeping grace fixed).
pub fn sweep_trust(pop: usize, ticks: usize, steps: usize, grace: f64) -> Vec<(f64, GraceResult)> {
    (0..steps).map(|i| {
        let t = i as f64 / (steps - 1).max(1) as f64;
        let p = GraceParams::new(pop, ticks).with_grace(grace).with_trust(t);
        (t, run(p, 42 + i as u64))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn params_default() { let p = GraceParams::new(100, 1000); assert!(p.grace_rate < 0.1); assert!(p.trust_rebuild > 0.3); }
    #[test] fn params_builder() { let p = GraceParams::new(100, 1000).with_grace(0.5).with_trust(0.8); assert_eq!(p.grace_rate, 0.5); assert_eq!(p.trust_rebuild, 0.8); }
    #[test] fn rng_same_seed() { let mut r1 = Rng::new(42); let mut r2 = Rng::new(42); assert_eq!(r1.next(), r2.next()); }
    #[test] fn run_basic() { let r = run(GraceParams::new(30, 100), 42); assert!(r.survival_rate >= 0.0 && r.survival_rate <= 1.0); }
    #[test] fn run_reproducible() { let p = GraceParams::new(50, 200); let r1 = run(p, 42); let r2 = run(p, 42); assert_eq!(r1.survival_rate, r2.survival_rate); }
    #[test] fn high_grace_kills() { let p = GraceParams::new(100, 500).with_grace(0.9).with_trust(0.1); let r = run(p, 42); assert!(r.survival_rate < 0.5, "high grace should kill, got {}", r.survival_rate); }
    #[test] fn low_grace_survives() { let p = GraceParams::new(100, 500).with_grace(0.01).with_trust(0.5); let r = run(p, 42); assert!(r.survival_rate > 0.2, "low grace should survive, got {}", r.survival_rate); }
    #[test] fn trust_helps() { let p_low = GraceParams::new(100, 500).with_grace(0.05).with_trust(0.1); let p_high = GraceParams::new(100, 500).with_grace(0.05).with_trust(0.8); let r_low = run(p_low, 42); let r_high = run(p_high, 42); assert!(r_high.trust_events >= r_low.trust_events); }
    #[test] fn entropy_range() { let r = run(GraceParams::new(100, 100), 42); assert!(r.entropy >= 0.0 && r.entropy <= 1.585); }
    #[test] fn gamma_range() { let r = run(GraceParams::new(100, 100), 42); assert!(r.mean_abs_gamma >= 0.0 && r.mean_abs_gamma <= 1.0); }
    #[test] fn events_tracked() { let r = run(GraceParams::new(100, 500), 42); assert!(r.trust_events + r.grace_events + r.tunnel_events + r.trap_events > 0); }
    #[test] fn sweep_grace_runs() { let results = sweep_grace(30, 100, 10, 0.5); assert_eq!(results.len(), 10); }
    #[test] fn sweep_trust_runs() { let results = sweep_trust(30, 100, 10, 0.05); assert_eq!(results.len(), 10); }
    #[test] fn peak_gte_final() { let r = run(GraceParams::new(50, 200), 42); assert!(r.peak_survival >= r.survival_rate); }
    #[test] fn no_tunnel_death() { let p = GraceParams::new(50, 1000).with_tunnel(0.0).with_grace(0.5).with_trust(0.1); let r = run(p, 42); assert!(r.survival_rate < 0.3, "no tunnel should kill with high grace"); }
}
