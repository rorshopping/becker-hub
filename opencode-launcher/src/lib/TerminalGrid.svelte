<script>
  import { tabs, activeTab, settings } from '../stores.js';
  import Terminal from './Terminal.svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  async function launchAll(tabId) {
    const tab = $tabs.find(t => t.id === tabId);
    if (!tab) return;
    for (let i = 0; i < 4; i++) {
      if (tab.slots[i]) continue;
      const cmd = tab.slotMeta?.[i]?.command || $settings.defaultCommand;
      const cwd = tab.slotMeta?.[i]?.cwd || null;
      const id = await invoke('spawn_cli', {
        tabId: tab.id,
        slot: i,
        command: cmd,
        cwd,
      });
      tabs.setSlot(tab.id, i, id);
      tabs.setSlotMeta(tab.id, i, { command: cmd, cwd });
    }
  }
</script>

<div class="grid-wrap">
  {#each $tabs as tab (tab.id)}
    <div class="tab-grid" class:active={$activeTab === tab.id}>
      {#if tab.slots.every(s => !s)}
        <div class="launch-overlay">
          <button class="launch-btn" on:click={() => launchAll(tab.id)}>
            ▶ Launch 4× {$settings.defaultCommand}
          </button>
        </div>
      {/if}
      <div class="grid">
        {#each [0,1,2,3] as i (tab.id + '-' + i)}
          <Terminal
            tabId={tab.id}
            slot={i}
            ptyId={tab.slots[i]}
            command={tab.slotMeta?.[i]?.command || $settings.defaultCommand}
            cwd={tab.slotMeta?.[i]?.cwd}
          />
        {/each}
      </div>
    </div>
  {/each}
</div>

<style>
  .grid-wrap { flex: 1; position: relative; min-height: 0; display: flex; flex-direction: column; }
  .tab-grid { display: none; flex: 1; min-height: 0; }
  .tab-grid.active { display: flex; flex-direction: column; flex: 1; min-height: 0; }
  .grid {
    display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr;
    gap: 4px; padding: 4px; height: 100%; background: #30363d; flex: 1;
  }
  .launch-overlay {
    position: absolute; inset: 0; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    background: rgba(13,17,23,0.85); z-index: 10; backdrop-filter: blur(6px);
  }
  .launch-btn {
    padding: 16px 32px; font-size: 16px; font-weight: 600;
    background: linear-gradient(135deg, #1f6feb, #388bfd);
    color: white; border: none; border-radius: 10px; cursor: pointer;
    box-shadow: 0 4px 24px rgba(31,111,235,0.4);
  }
  .launch-btn:hover { transform: translateY(-1px); }
</style>