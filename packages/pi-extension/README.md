# @fishword/pi-extension

Pi extension for Fishword. The extension calls the Rust CLI through
`@fishword/cli` and renders vocabulary cards and learning stats inside Pi
overlays.

## Commands

```text
/fw-deck      Switch active deck
/fw-stats     Show 7-day learning stats
/fw-again     Rate current card as again
/fw-hard      Rate current card as hard
/fw-good      Rate current card as good
/fw-easy      Rate current card as easy
```

## Shortcuts

```text
ctrl+shift+v  Refresh current card
ctrl+shift+a  Again
ctrl+shift+h  Hard
ctrl+shift+g  Good
ctrl+shift+e  Easy
```

The stats overlay uses `fishword status --json` and
`fishword stats --range 7d --json` as data sources, then draws the trend chart
in the Pi UI layer.
