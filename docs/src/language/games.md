# Supported Games

The `game='…'` binding selects the poker variant. Each variant changes the deck, the number of hole cards, and the hand evaluator.

| Value       | Variant            | Hole cards | Deck         |
| ----------- | ------------------ | ---------- | ------------ |
| `holdem`    | Texas Hold'em      | 2          | Full 52      |
| `omaha`     | Pot-Limit Omaha    | 4          | Full 52      |
| `omaha5`    | 5-Card Omaha (Big O) | 5        | Full 52      |
| `omaha6`    | 6-Card Omaha (PLO6) | 6         | Full 52      |
| `shortdeck` | Short-Deck Hold'em | 2          | 36 (6s–As)   |

`holdem` is the default if `game` is omitted. Open PQL is currently a **Hi-only** implementation — Hi/Lo splits (Omaha 8, Stud 8) and stud variants (Stud Hi, Razz) are not supported.

## Hold'em

Players are dealt two hole cards, share a five-card board, and use any combination of seven cards to make the best five-card hand.

```sql
select avg(equity(hero, river))
from   game='holdem', hero='AhKh', villain='QQ+', board='Jh9s2c'
```

## Omaha

Four, five, or six hole cards per player (`omaha`, `omaha5`, `omaha6`). Each player **must** use exactly two of their hole cards and three of the board cards. Range strings still use the same notation; concrete hands require as many cards as the variant deals (e.g. `AhAsKhKs` for `omaha`, plus one more card per player for `omaha5`, two more for `omaha6`):

```sql
select avg(equity(hero, river))
from   game='omaha', hero='AhAsKhKs', villain='**'
```

```sql
select avg(equity(hero, river))
from   game='omaha6', hero='AhAsKhKsQhQs', villain='**'
```

## Short Deck

A 36-card deck (deuces through fives removed). Common Short-Deck rule choices apply: A-6-7-8-9 is the wheel straight, and flushes beat full houses. The prelude crate's evaluator implements the standard ranking.

```sql
select avg(equity(hero, river))
from   game='shortdeck', hero='AwAx', villain='**'
```

## One Game per Query

Each query targets a single game. You cannot mix variants inside one query.
