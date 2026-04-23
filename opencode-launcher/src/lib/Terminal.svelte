<script>
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen } from '@tauri-apps/api/event';
  import { tabs, activeTab, recordScrollback, clearScrollback } from '../stores.js';
  import '@xterm/xterm/css/xterm.css';

  export let tabId;
  export let slot;
  export let ptyId;
  export let command;
  export let cwd = null;

  let container;
  let term;
  let fit;
  let unlistenData, unlistenExit;
  let focused = false;
  let autoLaunched = false;

  let currentPtyId = null;
  let inputHandler = null;
  let hasLaunched = false;

  async function attach(id) {
    console.log('[Terminal] attach:', id);
    unlistenData?.();
    unlistenExit?.();

    unlistenData = await listen(`pty-data:${id}`, (e) => {
      term.write(e.payload);
      recordScrollback(id, e.payload);
    });
    unlistenExit = await listen(`pty-exit:${id}`, () => {
      term.write('\r\n\x1b[33m[process exited]\x1b[0m\r\n');
      clearScrollback(id);
      tabs.setSlot(tabId, slot, null);
      currentPtyId = null;
    });
    currentPtyId = id;
  }

  async function launch() {
    const id = await invoke('spawn_cli', { tabId, slot, command, cwd: cwd || null });
    tabs.setSlot(tabId, slot, id);
    tabs.setSlotMeta(tabId, slot, { command, cwd });
    await attach(id);
    const dims = fit.proposeDimensions() ?? { cols: 80, rows: 24 };
    await invoke('resize_pty', { id, cols: dims.cols, rows: dims.rows });
  }

  onMount(async () => {
    console.log('[Terminal] mounted slot', slot, 'ptyId:', ptyId, 'tabId:', tabId);

    term = new Terminal({
      fontFamily: 'JetBrains Mono, Menlo, Consolas, monospace',
      fontSize: 12,
      theme: {
        background: '#0d1117', foreground: '#e6edf3', cursor: '#58a6ff',
        selectionBackground: '#1f6feb55',
      },
      cursorBlink: true,
      scrollback: 5000,
      disableStdin: false,
      allowTransparency: true,
    });

    inputHandler = (e) => {
      console.log('[Terminal] input:', e, 'currentPtyId:', currentPtyId);
      if (currentPtyId) {
        invoke('write_to_pty', { id: currentPtyId, data: e });
      }
    };

    fit = new FitAddon();
    term.loadAddon(fit);
    term.open(container);
    fit.fit();

    term.onData(inputHandler);
    term.textarea?.addEventListener('focus', () => focused = true);
    term.textarea?.addEventListener('blur', () => focused = false);

    const ro = new ResizeObserver(() => {
      try {
        fit.fit();
        if (currentPtyId) invoke('resize_pty', { id: currentPtyId, cols: term.cols, rows: term.rows });
      } catch {}
    });
    ro.observe(container);

    // Auto-launch if slot was restored from previous session
    if (ptyId === 'restored' && !hasLaunched) {
      hasLaunched = true;
      setTimeout(() => launch(), 100);
    } else if (ptyId && ptyId !== 'restored') {
      await attach(ptyId);
    }
  });

  

  onDestroy(() => {
    unlistenData?.(); unlistenExit?.();
    term?.dispose();
  });

  export function focus() { term?.focus(); }
</script>

<div class="pane" class:focused class:empty={!currentPtyId}>
  <div class="header">
    <span class="label">#{slot + 1}</span>
    {#if currentPtyId && currentPtyId !== 'restored'}<span class="status">● running</span>{/if}
    {#if ptyId === 'restored' && !currentPtyId}<span class="status restored">↻ click to launch</span>{/if}
  </div>
  <div class="term" bind:this={container}></div>
  {#if !currentPtyId}
    <div class="overlay">
      <button on:click={launch}>▶ Launch {command}</button>
    </div>
  {/if}
</div>

<style>
  .pane {
    position: relative; background: #0d1117; display: flex; flex-direction: column;
    border: 1px solid transparent; border-radius: 4px; overflow: hidden;
    height: 100%;
  }
  .pane.focused { border-color: #1f6feb; }
  .header {
    flex-shrink: 0;
    display: flex; justify-content: space-between; padding: 4px 8px;
    background: #161b22; font-size: 10px; color: #8b949e;
    border-bottom: 1px solid #30363d;
  }
  .status { color: #3fb950; }
  .status.restored { color: #d29922; }
  .term { flex: 1; min-height: 0; min-width: 0; display: flex; }
  .term :global(.xterm) { flex: 1; }
  .overlay {
    position: absolute; inset: 0; display: grid; place-items: center;
    background: rgba(13,17,23,0.6);
  }
  .overlay button {
    padding: 8px 16px; background: #21262d; color: #e6edf3;
    border: 1px solid #30363d; border-radius: 6px; cursor: pointer;
  }
  .overlay button:hover { border-color: #1f6feb; }
</style>
