<script>
  import { notifications } from '../stores.js';
  let expanded = false;
</script>

<div class="notifications">
  <button class="toggle" on:click={() => expanded = !expanded}>
    {expanded ? '▼' : '▲'} Notifications ({$notifications.length})
  </button>
  {#if expanded}
    <ul>
      {#each $notifications as n (n.id)}
        <li>
          <span class="msg">{n.snippet || 'Agent finished'}</span>
          <button on:click={() => notifications.remove(n.id)}>×</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .notifications {
    position: fixed; bottom: 0; left: 0; right: 0;
    background: #161b22; border-top: 1px solid #30363d;
    font-size: 12px;
  }
  .toggle {
    background: transparent; color: #8b949e; border: none;
    padding: 4px 12px; cursor: pointer; width: 100%; text-align: left;
  }
  ul { list-style: none; margin: 0; padding: 0; max-height: 200px; overflow-y: auto; }
  li {
    display: flex; justify-content: space-between; align-items: center;
    padding: 6px 12px; border-top: 1px solid #21262d;
  }
  .msg { color: #c9d1d9; font-family: monospace; font-size: 11px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    max-width: calc(100% - 40px); }
  li button {
    background: transparent; color: #8b949e; border: none;
    cursor: pointer; padding: 2px 6px;
  }
  li button:hover { color: #f85149; }
</style>