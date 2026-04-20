# Nova

Rust feature clone of [astropanel](https://github.com/isene/astropanel), a terminal panel for amateur astronomers.

Weather forecast + ephemeris + astronomical events. Built on crust. Shares the ephemeris engine with tock (currently copy/pasted; candidate for extraction to a shared crate).

## Build

```bash
PATH="/usr/bin:$PATH" cargo build --release
```

Note: `PATH` prefix needed to avoid `~/bin/cc` shadowing the C compiler.
