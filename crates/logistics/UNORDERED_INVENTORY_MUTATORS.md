# Known issue: two unordered inventory mutators

Status: **latent** — not broken today, cheap to fix before a second writer exists, near-impossible
to retrofit-debug after.

## The setup

Two systems mutate `Inventory`, both scheduled in Bevy's `Update`:

- `apply_commodity_transfers` (`src/systems.rs`) — drains `CommodityTransfer` messages,
  capacity-clamped deposits / unclamped withdrawals.
- `apply_cargo_hauls` (`src/cargo_handling/systems.rs`) — drains `CargoHaul` messages, moves goods
  source→dest.

In `src/plugin.rs` they're added like this:

```rust
.add_systems(Update, apply_commodity_transfers)
.add_systems(
    Update,
    (dock_haul_input.run_if(...), apply_cargo_hauls).chain(),
)
```

The `.chain()` only orders `dock_haul_input` **before** `apply_cargo_hauls`. There is **no ordering
edge** between `apply_commodity_transfers` and `apply_cargo_hauls`. Bevy is free to run them in either
order each frame.

## Why it doesn't bite today

1. **No shared target.** Right now `CommodityTransfer` only targets `Storage` (the debug keys `1–8`),
   and `CargoHaul` moves between storage and carrier. Within a single frame nothing writes both a
   transfer *and* a haul to the **same** entity. So whichever runs first, the end state is identical.
2. **No data race despite the unordered pair.** Both take `Query<&mut Inventory>`. Bevy's scheduler
   sees two systems with conflicting mutable access to the same component and refuses to run them
   simultaneously — it serializes them. So this is *not* a memory-safety or panic risk. It's purely a
   **determinism** risk.

## What would bite

The moment a second writer drives transfers at an entity that's also a haul endpoint in the same
frame. Concrete future case from the plan ("production drives transfers", "real order UI"):

> A production building deposits 10 grain into the carrier via `CommodityTransfer` **and** the carrier
> hauls its load into a storage via `CargoHaul`, same frame.

- Order A (transfer→haul): carrier gets +10 grain, then hauls grain-including-the-new-10 out.
- Order B (haul→transfer): carrier hauls out its old load, then receives +10 grain that sits until
  next frame.

Same inputs, two different world states, decided by scheduler whim. The classic ECS ordering bug:
invisible until the frame where both fire, then it's an intermittent "why did my cargo numbers
flicker" that's miserable to reproduce.

There's a subtler variant: a deposit clamped by `Capacity` depends on current fill. If a haul empties
the carrier first, the deposit fits; if not, it's partially rejected and the remainder is **silently
dropped** (transfers have no source to leave a remainder in, unlike hauls). So ordering doesn't just
shuffle timing — it can change *how much* exists.

## The fix (small)

Give the two an explicit order so the resolution is a decision, not an accident. Either chain them:

```rust
.add_systems(Update, (apply_commodity_transfers, apply_cargo_hauls).chain())
```

or, cleaner as this grows, a named `SystemSet` like `InventoryMutation` with an internal order, so
future inventory writers slot into a defined sequence instead of each new system re-opening the
question.

The real design question the fix forces: **within a frame, do transfers settle before hauls, or
after?** Pick deliberately. Lean: transfers-then-hauls (production/input lands, *then* it can be
moved), but it depends on whether goods should be haulable the same frame they're produced or only
the next.
