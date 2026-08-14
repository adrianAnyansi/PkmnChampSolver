# Data Caching Plan

## Goal
Keep Pokemon, moves, and (future) ability data in static/shared memory after first use, and avoid initializing every Pokemon at startup.

## Current Creation Paths

### Pokemon
- Pokemon data is loaded from embedded JSON in `src/pokemon.rs` via `include_str!("data/pokemon.json")`.
- `POKEMON_HASH` is a global `Lazy<HashMap<PokemonName, Pokemon>>`.
- `get_stat_json()` deserializes the entire pokemon JSON into a map, then deserializes each recognized species into `Pokemon`.
- `get_pkmn()` reads from that full map and returns `&'static Pokemon`.

### Moves
- Moves are built in `src/pokemon/moves.rs` by `get_move(PokemonMoveName) -> PokemonMove`.
- Every call creates a fresh `PokemonMove` using builder methods and allocates vectors for effects.

### Abilities
- `PokemonAbilityName` is currently an enum (name/id for abilities).
- `PokemonAbility` will be a data struct (parallel to how `PokemonMove` pairs with `PokemonMoveName`).
- Abilities are parsed during Pokemon deserialization and stored as enum values in each `Pokemon`.
- Active battle state stores one ability enum per active Pokemon.

## Recommended Architecture

Use two-level lazy caching:

1. Raw source index (cheap, one-time)
- Keep raw JSON values keyed by `PokemonName`.
- Example static:
  - `RAW_POKEMON: Lazy<HashMap<PokemonName, serde_json::Value>>`

2. Materialized object cache (on-demand)
- Deserialize/build only when requested.
- Store created objects in global cache maps.

## Pokemon Plan

- Add:
  - `POKEMON_CACHE: Lazy<RwLock<HashMap<PokemonName, Arc<Pokemon>>>>`
- Update API:
  - `get_pkmn(name) -> Arc<Pokemon>` (or add `get_pkmn_arc` first for migration)
- Resolution flow:
  1. Try cache read lock.
  2. If missing, read raw JSON for that species.
  3. Deserialize that single Pokemon.
  4. Insert into cache and return cloned `Arc`.

Result: only used species are deserialized.

## Move Plan

- Keep current match-based move builder as `build_move(name) -> PokemonMove`.
- Add:
  - `MOVE_CACHE: Lazy<RwLock<HashMap<PokemonMoveName, Arc<PokemonMove>>>>`
- Update API:
  - `get_move(name) -> Arc<PokemonMove>`
- Flow:
  1. Lookup cache.
  2. If missing, build once and cache.
  3. Return cloned `Arc`.

Result: move definitions are created once and reused.

## Ability Plan

- Short term: keep `PokemonAbilityName` enum as-is (already cheap).
- Long term (if ability logic grows): create `AbilityData` + lazy cache/registry similar to moves.

## Arc Explained for This Project

`Arc<Pokemon>` means a shared, reference-counted pointer to one `Pokemon` object.

- `Arc` is an atomic reference-counted smart pointer.
- Cloning an `Arc` is cheap: it increments a counter instead of copying the whole `Pokemon` data.
- Memory is automatically freed when the last `Arc` is dropped.

Why this fits your use-case:

- You want objects to live in a shared/static area after first creation.
- You only want to initialize species/moves/abilities when they are actually used.
- `Arc` supports "create once, share many times" without forcing `'static` leaks.

How this would work for Pokemon:

1. Request a species (for example `Garchomp`).
2. Check `POKEMON_CACHE` for `PokemonName::Garchomp`.
3. If missing, deserialize only that entry from the raw JSON index.
4. Wrap in `Arc<Pokemon>`, cache it, and return a cloned `Arc` handle.
5. All consumers share the same underlying object.

Why this is better than `&'static Pokemon` for lazy loading:

- `&'static` strongly pushes design toward eager global init (or leaking boxed data).
- `Arc` owns the object and avoids difficult lifetime constraints.
- You still get singleton-like behavior after first load, but only for requested entries.

Rule of thumb for this codebase:

- Immutable shared definitions (`Pokemon`, `PokemonMove`, ability data struct): `Arc<T>`.
- Shared mutable global caches: `RwLock<HashMap<..., Arc<T>>>`.

## Type and Lifetime Migration

Current `ActivePokemon` stores `&'static Pokemon`.
To support true on-demand cache without leaking memory:

- Change `ActivePokemon.pokemon` to `Arc<Pokemon>`.
- Update constructors and call sites to pass/clone `Arc<Pokemon>`.
- Update move action ownership similarly if needed (`Arc<PokemonMove>` in action data, or borrow from a cached owner).

This avoids requiring `'static` references for objects created lazily at runtime.

## Why This Solves the Problem

- No full Pokemon object initialization at startup.
- Data still lives in a shared static/global area after first request.
- Repeated accesses are cheap due to cache hits.
- Architecture scales as more species/moves/ability logic are added.
