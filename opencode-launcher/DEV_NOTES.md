# Development Notes

## Svelte Version Compatibility

**Critical**: This project uses **Svelte 4** syntax. Do NOT upgrade to Svelte 5.

When setting up or updating dependencies:
- Use `svelte: ^4.2.0` (not ^5.x)
- Use `@sveltejs/vite-plugin-svelte: ^3.0.0` (not ^4.x or ^7.x)
- Use `vite: ^5.0.0` (not ^8.x)

Svelte 5 changed component API significantly (no more `export let`, different event syntax). If you see blank screens or compilation errors, verify the versions in `package.json`.

## Common Svelte 4 Patterns

### Props
```svelte
<script> export let name; </script>
```

### Events
```svelte
on:click={handler}
on:click|stopPropagation={handler}
on:keydown|stopPropagation={(e) => e.key === 'Enter' && handler()}
```

### Classes
```svelte
class:active={condition}
class:name-with-underscores={condition}
```

## xterm.js Terminal Styling

For proper terminal rendering in a flex container:
```svelte
<style>
  .pane { height: 100%; display: flex; flex-direction: column; }
  .header { flex-shrink: 0; }
  .term { flex: 1; min-height: 0; min-width: 0; display: flex; }
  .term :global(.xterm) { flex: 1; }
</style>
```

Key points:
- Parent must have `height: 100%`
- Header needs `flex-shrink: 0`
- Terminal container needs `min-height: 0` for flex to work
- Target `.xterm` directly for proper sizing

## Accessibility (a11y) Warnings

Svelte enforces accessibility checks. Common patterns to avoid warnings:

### Non-interactive elements with click handlers
```svelte
<!-- Wrong -->
<span on:click={handler}>text</span>

<!-- Correct -->
<span on:click={handler} on:keydown={(e) => e.key === 'Enter' && handler()} role="button" tabindex="0">text</span>
```

### Better: Use buttons instead
```svelte
<button class="close" on:click|stopPropagation={handler}>×</button>
```

## Voice Input

Uses Web Speech API (browser-native). No native/Rust setup required.

```javascript
const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
if (!SpeechRecognition) { /* not supported */ }
```

Note: Chromium-based browsers (Chrome, Edge, Brave) support this. Firefox may not.

## Rust/Cargo Notes

### Dead Code Warnings
Use `#[allow(dead_code)]` on structs/methods that are defined for future use or serialization:
```rust
#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
struct PersistedTab { ... }
```

### Struct Definitions
When using `impl Type` blocks, ensure the struct is defined:
```rust
pub struct PtyManager { ... }

impl PtyManager { ... }  // must come after struct definition
```

## Keyboard Shortcuts (TODO)

- `Ctrl+L` - Clear terminal
- `Ctrl+T` - New tab
- `Ctrl+W` - Close current terminal
- `Ctrl+1-4` - Switch to slot 1-4

## Terminal Double-Key Issue

If typing produces double characters (e.g., "tt" instead of "t"), the cause is likely:
1. A reactive statement (`$:`) calling `attach()` when `ptyId` changes
2. AND `attach()` already being called from `onMount` via `launch()`

This causes two event listeners to be registered, each sending input to the PTY.

**Fix**: Remove reactive statements that duplicate `attach()` calls. The `launch()` function already calls `attach()`, so let the flow be:
- `onMount` → `launch()` → `attach()` → done

Do NOT add `$: if (ptyId) attach(ptyId)` reactive statements - they will cause double-attaching.

## PTY Command Execution (Windows)

Use `cmd.exe /c` for single command execution that doesn't double-echo:
```rust
let mut c = CommandBuilder::new("cmd.exe");
c.args(["/c", &format!("{} & pause", command)]);
```

Avoid PowerShell `-Command` mode for simple commands - it can cause double echo issues.

## Multi-Tab Terminal Management

When rendering multiple tabs with terminal instances (xterm.js), Svelte's component lifecycle causes issues:

### Problem: Tabs Share Same Terminal Instances
Using `{#if current}` to conditionally render terminals causes Svelte to reuse components incorrectly across tabs.

### Solution: Render All Tabs, Show Only Active
```svelte
{#each $tabs as tab (tab.id)}
  <div class="tab-grid" class:active={$activeTab === tab.id}>
    <div class="grid">
      {#each [0,1,2,3] as i (tab.id + '-' + i)}
        <Terminal tabId={tab.id} slot={i} ptyId={tab.slots[i]} />
      {/each}
    </div>
  </div>
{/each}

<style>
  .tab-grid { display: none; }
  .tab-grid.active { display: flex; }
</style>
```

### Key Points:
1. Always use unique keys: `(tab.id + '-' + i)` to distinguish terminals
2. Render all tabs but hide inactive ones with CSS (`display: none`)
3. This preserves terminal state and prevents component reuse bugs
4. Svelte's `{#if}` with reactive data causes component reuse issues

## Session Restore Best Practices

### Don't Store Scrollback in Session
Storing scrollback in session.json causes issues:
- Large file sizes
- UTF-8 byte boundary corruption when truncating

### Session Restore Pattern
```javascript
// Mark slots as needing launch instead of storing state
slots: ptab.slots.map(s => s ? 'restored' : null),
```

Then auto-launch on mount:
```javascript
if (ptyId === 'restored' && !hasLaunched) {
  setTimeout(() => launch(), 100);
}
```

### Clear Old Session Artifacts
Delete `%LOCALAPPDATA%\opencode-launcher\session.json` if corrupted scrollback causes issues.