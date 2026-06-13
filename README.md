# Ternary Grace — Grace vs Trust Rebuild: Two Opposing Forces in Agent Dynamics

**Ternary Grace** separates two often-conflated parameters in agent conflict resolution: **grace** (willingness to pause and reconsider — enters the spindle state 0) and **trust rebuild** (willingness to cooperate after conflict — exits the spindle). These are opposing forces: high grace pushes agents into the neutral state, while high trust rebuild pulls them out. The crate simulates the interaction of these forces and identifies the parameter regions where fleets survive vs. collapse.

## Why It Matters

Most agent conflict models conflate "entering disagreement" and "recovering from disagreement" into a single parameter. This is wrong. The distinction matters enormously: high grace + low trust rebuild means every disagreement sends agents to state 0 and they never leave — population collapse. Low grace + high trust rebuild means agents hold their ground but reconnect quickly — robust diversity. This crate provides the experimental framework to discover these phase boundaries and identify the parameter combinations that produce stable, adaptive ternary agent populations. The findings directly inform fleet configuration: how much forgiveness (trust rebuild) and caution (grace) to build into the system.

## How It Works

### Two-Parameter Model

The simulation tracks N agents, each with:
- **state** ∈ {-1, 0, +1}: current ternary position
- **trust** ∈ [0, 255]: how much the agent trusts others (starts at 128)

Each tick:
1. **Grace check**: If an agent encounters disagreement and `random() < grace_rate`, it enters state 0 (spindle). Costly.
2. **Trust rebuild**: If an agent is in state 0 and `random() < trust_rebuild`, it transitions to +1 or -1 based on majority influence. Beneficial.
3. **Tunnel**: Base probability `tunnel_rate` of escaping 0 spontaneously.
4. **Trap**: Base probability `trap_rate` of falling into 0 from ±1.

### Phase Space Analysis

By sweeping `grace_rate` and `trust_rebuild` independently:

- **Survival region**: Low grace (< 0.1), moderate trust (> 0.3) → stable diversity
- **Grace death**: High grace (> 0.2), low trust (< 0.2) → everyone enters 0, never leaves
- **Rigid survival**: Very low grace (< 0.02), any trust → agents hold ground, robust but inflexible
- **Adaptive optimum**: Moderate both (~0.05 grace, ~0.5 trust) → agents pause when needed but recover

### Metrics

The `GraceResult` struct tracks:
- `survival_rate`, `entropy`, `mean_abs_gamma`: population health
- `trust_events`, `grace_events`, `tunnel_events`, `trap_events`: process counters
- `collapse_tick`: when survival first drops below 0.1

## Quick Start

```rust
use ternary_grace::{GraceParams, run};

// Test the danger zone: high grace, low trust
let params = GraceParams::new(300, 1000)
    .with_grace(0.2)      // 20% chance of entering spindle on disagreement
    .with_trust(0.05);    // 5% chance of rebuilding trust per tick

let result = run(params, 42);
println!("Survival: {:.1}%", result.survival_rate * 100.0);
println!("Grace events: {}, Trust events: {}", result.grace_events, result.trust_events);

// Compare with safe parameters
let safe = GraceParams::new(300, 1000)
    .with_grace(0.03)
    .with_trust(0.5);
let safe_result = run(safe, 42);
```

```bash
cargo add ternary-grace
```

## API

| Type / Function | Description |
|---|---|
| `GraceParams` | `new(pop, ticks)`, `.with_grace()`, `.with_trust()`, `.with_tunnel()`, `.with_trap()` |
| `run(GraceParams, seed) → GraceResult` | Run simulation |
| `GraceResult` | Survival, entropy, event counts, collapse tick |

## Architecture Notes

Grace vs trust is a critical design parameter in **SuperInstance** fleet configuration. Grace maps to the η term (entering entropy/uncertainty) and trust rebuild maps to γ (recovering growth/productivity). The γ + η = C conservation law predicts that the sum is bounded — the question is whether the system finds a dynamic equilibrium or spirals into collapse. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Axelrod, Robert. *The Evolution of Cooperation*, Basic Books, 1984 — trust and forgiveness in iterated games.
- Nowak, Martin & Sigmund, Karl. "A Strategy of Win-Stay, Lose-Shift," *Nature*, 364, 1993.
- Deutsch, Morton. *The Resolution of Conflict*, Yale UP, 1973 — trust and conflict theory.

## License

MIT
