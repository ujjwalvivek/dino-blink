# Dino Blink 🦕⚡

[![npm](https://img.shields.io/npm/v/@ujjwalvivek/dino-blink?style=for-the-badge&color=95c85a)](https://www.npmjs.com/package/@ujjwalvivek/dino-blink)
[![GitHub release](https://img.shields.io/github/v/release/ujjwalvivek/dino-blink?style=for-the-badge&color=7fc4b8)](https://github.com/ujjwalvivek/dino-blink/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT?style=for-the-badge&color=fa983a)](LICENSE)

A retro-style infinite runner built on Journey Engine. The Engine is open sourced at [github.com/ujjwalvivek/journey-engine](https://github.com/ujjwalvivek/journey-engine).

## Installation

```bash
npm install @ujjwalvivek/dino-blink@${version}
```

## Usage

### Quick Embed

```html
<script type="module">
  import init from 'https://cdn.jsdelivr.net/npm/@ujjwalvivek/dino-blink@${version}/dino_blink.js';
  await init();
</script>
```

### npm / bundler

```javascript
import init from '@ujjwalvivek/dino-blink@${version}';
await init();
```

### React / Framework

```typescript
import { useEffect } from 'react';
import init from '@ujjwalvivek/dino-blink@${version}';

export function DinoGame() {
  useEffect(() => { init(); }, []);
  return <div style={{ width: '100%', height: '100vh' }} />;
}
```

## Building locally

Requires Rust + [wasm-pack](https://rustwasm.github.io/wasm-pack/).

```bash
# build WASM into pkg/
wasm-pack build --target web --scope ujjwalvivek

# serve locally (browsers block WASM from file://)
npx serve .
```

## Technical Details

- **Size:** 6MB (WASM binary + JS bindings)
- **LOC**: 500 lines of Code
- **Engine:** Journey Engine (custom Rust game framework)
- **Target:** WebAssembly (ES6 modules)

## License

MIT

---

**Play it live:** [ujjwalvivek.itch.io/dino-blink](https://ujjwalvivek.itch.io/dino-blink)  
**Source:** [github.com/ujjwalvivek/dino-blink](https://github.com/ujjwalvivek/dino-blink)
