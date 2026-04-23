<script>
  import { onMount, onDestroy } from 'svelte';
  import TabBar from './lib/TabBar.svelte';
  import TerminalGrid from './lib/TerminalGrid.svelte';
  import NotificationPane from './lib/NotificationPane.svelte';
  import VoiceInput from './lib/VoiceInput.svelte';
  import { tabs, activeTab, notifications, settings, sessionManager } from './stores.js';
  import { listen } from '@tauri-apps/api/event';
  import { appWindow } from '@tauri-apps/api/window';

  let unlistenFinished;
  let saveInterval;
  let voiceInput;

  function handleKeydown(e) {
    console.log('[Key] ctrl:', e.ctrlKey, 'shift:', e.shiftKey, 'key:', e.key);
    if (e.ctrlKey && e.shiftKey && e.key === 'V') {
      e.preventDefault();
      console.log('[Key] Calling voice toggle');
      voiceInput?.toggle();
    }
  }

  onMount(async () => {
    // Restore previous session
    await sessionManager.restore();

    // If no tabs were restored, create a fresh one
    const currentTabs = sessionManager.getCurrentTabs();
    if (currentTabs.length === 0) {
      tabs.addTab();
    }

    unlistenFinished = await listen('agent-finished', (e) => {
      notifications.add(e.payload);
    });

    // Auto-save session every 30 seconds
    saveInterval = setInterval(() => {
      sessionManager.save();
    }, 30000);

    // Save on window close
    await appWindow.onCloseRequested(async (event) => {
      await sessionManager.save();
    });
  });

  onDestroy(() => {
    unlistenFinished?.();
    if (saveInterval) clearInterval(saveInterval);
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<main>
  <header class="toolbar">
    <h1>Becker Hub</h1>
    <VoiceInput bind:this={voiceInput} />
  </header>
  <div class="workspace">
    <TabBar />
    <TerminalGrid />
  </div>
  <NotificationPane />
</main>

<style>
  :global(body) {
    margin: 0; background: #0d1117; color: #e6edf3;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", Inter, sans-serif;
    overflow: hidden;
  }
  main { display: flex; flex-direction: column; height: 100vh; }
  .toolbar {
    display: flex; justify-content: space-between; align-items: center;
    padding: 8px 16px; background: #161b22; border-bottom: 1px solid #30363d;
    -webkit-app-region: drag;
  }
  .toolbar h1 { font-size: 14px; font-weight: 600; margin: 0; letter-spacing: 0.3px; }
  .workspace { flex: 1; display: flex; flex-direction: column; min-height: 0; }
</style>
