# ternary-grace

**Grace vs. Trust Rebuild: two opposing forces, not one parameter.**

This is the crate that discovered something unexpected. We started with a simple question: *should agents forgive?* The answer turned out to be **no, not really** — at least not the way we thought.

The insight: there are two *different* mechanisms at work:
- **Grace** = willingness to pause and reconsider. You enter the spindle (the 0 state). This is *costly* — you stop contributing.
- **Trust rebuild** = willingness to cooperate after conflict. You reconnect. This is *beneficial* — you resume productive interaction.

High grace + low trust = everyone pauses, nobody reconnects = death. The 2D parameter sweep proved it: **forgiveness kills, trust heals.**

## What's Inside

- **`GraceParams`** — configure `grace_rate` (keep LOW), `trust_rebuild` (keep MODERATE), `tunnel_rate`, `trap_rate`
- **`GraceResult`** — survival rate, entropy, collapse tick, plus counts of grace/trust/tunnel/trap events
- **`run(params, seed)`** — single simulation
- **`sweep_2d(grace_range, trust_range, seeds)`** — the definitive experiment: sweep both parameters, find the survival boundary
- **`optimal_params()`** — the empirically discovered safe defaults

## Quick Example

```rust
use ternary_grace::*;

// Safe parameters: low grace, moderate trust
let params = GraceParams::new(100, 1000)
    .with_grace(0.05)      // rarely enter spindle
    .with_trust(0.5)       // reconnect after conflict
    .with_tunnel(0.006);

let result = run(params, 42);
println!("Survival: {:.1}%", result.survival_rate * 100.0);
println!("Trust events: {}", result.trust_events);
println!("Grace events: {}", result.grace_events);

// THE experiment: 2D sweep
let grace_vals: Vec<f64> = (0..20).map(|i| i as f64 * 0.05).collect();
let trust_vals: Vec<f64> = (0..20).map(|i| i as f64 * 0.05).collect();
let sweep = sweep_2d(&grace_vals, &trust_vals, 10);
// Find: grace is ALWAYS costly, trust is ALWAYS beneficial
// The spindle is NOT for forgiveness — it's a trap
```

## The Insight

**The 0 state is a universal screen, and entering it is always costly.** The 2D sweep across 400 parameter combinations proved: no matter what other parameters you set, higher grace (more willingness to enter the spindle) always reduces survival, and higher trust rebuild always increases it. The protocol: *never deliberately enter the spindle; maintain trust to tunnel out faster.*

This has implications beyond agent systems. In any system with a "pause and reconsider" state, the lesson applies: pausing is expensive. Reconnecting is cheap. Optimize for reconnection, not reflection.

**Use cases:**
- **Multi-agent systems** — configure trust/grace parameters for fleet survival
- **Organization design** — model conflict resolution strategies
- **Relationship modeling** — the math of forgiveness vs. trust
- **Diplomacy simulation** — grace periods vs. trust rebuilding in negotiations
- **Network protocols** — timeout (grace) vs. reconnection (trust) strategies

## See Also

- **ternary-experiment** — the parameter sweep framework used for the 2D findings
- **ternary-cell** — the 3-byte cell that stores grace/trust state at scale
- **ternary-predict** — prediction-first approach to avoiding the spindle entirely
- **ternary-speculate** — simulate trust rebuilds before committing

## Install

```bash
cargo add ternary-grace
```

## License

MIT
