<script>
  import { tabs, activeTab } from '../stores.js';

  let editing = null;
  let editValue = '';

  function startRename(tab) {
    editing = tab.id;
    editValue = tab.name;
  }

  function finishRename(tabId) {
    if (editValue.trim()) tabs.renameTab(tabId, editValue.trim());
    editing = null;
  }

  function handleKeydown(e, tabId) {
    if (e.key === 'Enter') finishRename(tabId);
    if (e.key === 'Escape') editing = null;
  }
</script>

<div class="tabbar">
  {#each $tabs as tab}
    <button
      class="tab"
      class:active={$activeTab === tab.id}
      on:click={() => activeTab.set(tab.id)}
      on:dblclick={() => startRename(tab)}
    >
      {#if editing === tab.id}
        <input
          class="edit-input"
          bind:value={editValue}
          on:blur={() => finishRename(tab.id)}
          on:keydown={(e) => handleKeydown(e, tab.id)}
          autofocus
        />
      {:else}
        <span class="name">{tab.name}</span>
      {/if}
      <span class="slots">
        {#each tab.slots as s}
          <i class="dot" class:on={!!s}></i>
        {/each}
      </span>
      <span class="close" on:click|stopPropagation={() => tabs.removeTab(tab.id)} on:keydown|stopPropagation={(e) => e.key === 'Enter' && tabs.removeTab(tab.id)} role="button" tabindex="0">×</span>
    </button>
  {/each}
  <button class="add" on:click={() => tabs.addTab()}>+ Tab</button>
</div>

<style>
  .tabbar {
    display: flex; gap: 4px; padding: 6px 8px;
    background: #0d1117; border-bottom: 1px solid #30363d;
    overflow-x: auto;
  }
  .tab {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 12px; border-radius: 6px 6px 0 0;
    background: #161b22; border: 1px solid #30363d; border-bottom: none;
    color: #8b949e; cursor: pointer; font-size: 12px;
  }
  .tab.active { background: #1f6feb22; color: #e6edf3; border-color: #1f6feb; }
  .name { max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .edit-input { background: #0d1117; border: 1px solid #1f6feb; color: #e6edf3; font-size: 12px; padding: 2px 4px; width: 100px; border-radius: 3px; }
  .slots { display: flex; gap: 2px; }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: #30363d; }
  .dot.on { background: #3fb950; }
  .close { color: #8b949e; padding: 0 4px; cursor: pointer; }
  .close:hover { color: #f85149; }
  .add { background: transparent; border: 1px dashed #30363d; color: #8b949e;
    border-radius: 6px; padding: 4px 12px; cursor: pointer; }
</style>