import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

function createTabs() {
  const { subscribe, update, set } = writable([]);
  return {
    subscribe,
    addTab(existingTab = null) {
      const id = existingTab?.id || crypto.randomUUID();
      const tab = existingTab || {
        id,
        name: `Session ${Math.floor(Math.random() * 999)}`,
        slots: [null, null, null, null],
        // Track command & cwd per slot for session restore
        slotMeta: [{}, {}, {}, {}],
      };
      // Ensure slotMeta exists
      if (!tab.slotMeta) tab.slotMeta = [{}, {}, {}, {}];
      update(ts => [...ts, tab]);
      activeTab.set(id);
      return id;
    },
    removeTab(id) {
      const t = get({ subscribe }).find(x => x.id === id);
      if (t) t.slots.forEach(s => s && invoke('kill_pty', { id: s }));
      update(ts => ts.filter(t => t.id !== id));
    },
    renameTab(id, name) {
      update(ts => ts.map(t => t.id === id ? { ...t, name } : t));
    },
    setSlot(tabId, slot, ptyId) {
      update(ts => ts.map(t => t.id === tabId
        ? { ...t, slots: t.slots.map((s, i) => i === slot ? ptyId : s) }
        : t));
    },
    setSlotMeta(tabId, slot, meta) {
      update(ts => ts.map(t => {
        if (t.id !== tabId) return t;
        const slotMeta = [...(t.slotMeta || [{}, {}, {}, {}])];
        slotMeta[slot] = { ...slotMeta[slot], ...meta };
        return { ...t, slotMeta };
      }));
    },
    setAll(tabsArr) {
      set(tabsArr);
    },
    getAll() {
      return get({ subscribe });
    },
  };
}

export const tabs = createTabs();
export const activeTab = writable(null);

function createNotifications() {
  const { subscribe, update } = writable([]);
  return {
    subscribe,
    add(n) { update(xs => [{ ...n, ts: Date.now() }, ...xs].slice(0, 50)); },
    remove(id) { update(xs => xs.filter(x => x.id !== id)); },
    clear() { update(() => []); },
  };
}
export const notifications = createNotifications();

export const settings = writable({
  defaultCommand: 'opencode',
  whisperModel: '',
});

// ──────────────────────────────────────────────
// Session persistence manager
// ──────────────────────────────────────────────

// Per-terminal scrollback buffer stored in JS (fed from pty-data events)
const scrollbackBuffers = new Map();
const MAX_SCROLLBACK = 8192;

export function recordScrollback(ptyId, chunk) {
  let buf = scrollbackBuffers.get(ptyId) || '';
  buf += chunk;
  if (buf.length > MAX_SCROLLBACK) buf = buf.slice(buf.length - MAX_SCROLLBACK);
  scrollbackBuffers.set(ptyId, buf);
}

export function getScrollback(ptyId) {
  return scrollbackBuffers.get(ptyId) || '';
}

export function clearScrollback(ptyId) {
  scrollbackBuffers.delete(ptyId);
}

export const sessionManager = {
  async save() {
    try {
      const allTabs = tabs.getAll();
      const currentActiveTab = get(activeTab);
      const currentSettings = get(settings);

      const persistedTabs = allTabs.map(tab => ({
        id: tab.id,
        name: tab.name,
        slots: tab.slots.map((ptyId, i) => {
          if (!ptyId) return null;
          const meta = tab.slotMeta?.[i] || {};
          return {
            command: meta.command || currentSettings.defaultCommand,
            cwd: meta.cwd || null,
            scrollback: getScrollback(ptyId),
          };
        }),
      }));

      const sessionData = {
        tabs: persistedTabs,
        active_tab: currentActiveTab,
        settings: {
          default_command: currentSettings.defaultCommand,
          whisper_model: currentSettings.whisperModel,
        },
      };

      await invoke('save_session', { data: JSON.stringify(sessionData, null, 2) });
      console.log('[session] saved', persistedTabs.length, 'tabs');
    } catch (e) {
      console.error('[session] save failed:', e);
    }
  },

  async restore() {
    try {
      const raw = await invoke('load_session');
      if (!raw || raw === 'null') {
        console.log('[session] no previous session found');
        return;
      }

      const session = JSON.parse(raw);
      console.log('[session] restoring', session.tabs?.length, 'tabs');

      // Restore settings
      if (session.settings) {
        settings.set({
          defaultCommand: session.settings.default_command || 'opencode',
          whisperModel: session.settings.whisper_model || '',
        });
      }

      // Restore tabs
      if (session.tabs && session.tabs.length > 0) {
        tabs.setAll([]);
        activeTab.set(null);

        for (const ptab of session.tabs) {
          const tab = {
            id: ptab.id,
            name: ptab.name,
            slots: ptab.slots.map(s => s ? 'restored' : null), // Mark as needing launch
            slotMeta: ptab.slots.map(s => s ? { command: s.command, cwd: s.cwd } : {}),
          };
          tabs.addTab(tab);
        }

        if (session.active_tab) {
          activeTab.set(session.active_tab);
        }
      }
    } catch (e) {
      console.error('[session] restore failed:', e);
    }
  },

  getCurrentTabs() {
    return tabs.getAll();
  },
};
