<script>
  import { invoke } from '@tauri-apps/api/tauri';
  import { tabs, activeTab } from '../stores.js';

  let recording = false;
  let status = '';
  let error = '';
  let recognition;
  let stream;

  function checkSupport() {
    const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SR) {
      error = 'Speech recognition not supported. Use Chrome, Edge, or Brave.';
      return false;
    }
    return true;
  }

  function initRecognition() {
    const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
    recognition = new SR();
    recognition.continuous = false;
    recognition.interimResults = false;
    recognition.lang = 'en-US';

    try {
      const gl = new webkitSpeechGrammarList();
      gl.addFromString('#JSGF V1.0; grammar commands; public command = opencode | claude code | cursor | code | terminal | tab;', 1);
      recognition.grammars = gl;
    } catch (e) {}

    recognition.onstart = () => {
      status = 'Listening...';
      error = '';
    };

    recognition.onresult = (e) => {
      const text = e.results[0][0].transcript;
      console.log('[Voice] Recognized:', text);
      sendToActive(text);
      status = 'Sent!';
      setTimeout(() => { status = ''; }, 2000);
    };

    recognition.onerror = (e) => {
      console.error('[Voice] Error:', e.error);
      switch (e.error) {
        case 'not-allowed':
          error = 'Microphone denied. Check browser permissions.';
          break;
        case 'no-speech':
          error = 'No speech detected. Try again.';
          break;
        case 'network':
          error = 'Network error. Check internet connection.';
          break;
        case 'aborted':
          error = '';
          break;
        default:
          error = 'Error: ' + e.error;
      }
      recording = false;
    };

    recognition.onend = () => {
      recording = false;
      if (!error) status = '';
    };
  }

  export async function toggle() {
    console.log('[Voice] toggle called');
    error = '';

    if (recording) {
      recognition?.stop();
      recording = false;
      status = '';
      return;
    }

    if (!recognition) {
      if (!checkSupport()) return;
      initRecognition();
    }

    try {
      status = 'Starting...';
      await recognition.start();
      recording = true;
    } catch (e) {
      console.error('[Voice] Start failed:', e);
      error = 'Failed to start: ' + e.message;
      recording = false;
    }
  }

  function sendToActive(text) {
    // Fix common misrecognitions: "cloud code" -> "claude code"
    let corrected = text.toLowerCase()
      .replace(/\bcloud code\b/g, 'claude code')
      .replace(/\bclaude code\b/g, 'claude code')
      .replace(/\bclowd code\b/g, 'claude code')
      .replace(/\bcloud\b/g, 'opencode')
      .replace(/\bclaude\b/g, 'claude code');
    
    const tab = $tabs.find(t => t.id === $activeTab);
    if (!tab) {
      console.warn('[Voice] No active tab');
      return;
    }
    const target = tab.slots.find(Boolean);
    if (target) {
      console.log('[Voice] Sending to PTY:', target, corrected);
      invoke('write_to_pty', { id: target, data: corrected + '\n' }).catch(e => {
        console.error('[Voice] Write failed:', e);
      });
    } else {
      console.warn('[Voice] No running PTY in active tab');
    }
  }
</script>

<div class="voice">
  <button
    id="voice-btn"
    class:recording
    class:error={!!error}
    on:click={toggle}
    title="Voice input (Ctrl+Shift+V)"
  >
    {#if recording}
      ● Stop
    {:else if error}
      ⚠ Voice
    {:else if status}
      {status}
    {:else}
      🎤 Voice
    {/if}
  </button>
  {#if error}
    <span class="error-text">{error}</span>
  {/if}
</div>

<style>
  .voice {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .voice button {
    background: #21262d;
    border: 1px solid #30363d;
    color: #e6edf3;
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    -webkit-app-region: no-drag;
    min-width: 80px;
  }
  .voice button.recording {
    background: #da363366;
    border-color: #f85149;
    animation: pulse 1s infinite;
  }
  .voice button.error {
    border-color: #f85149;
  }
  .voice button:hover:not(.recording) {
    border-color: #1f6feb;
  }
  @keyframes pulse {
    50% { opacity: 0.6; }
  }
  .error-text {
    font-size: 11px;
    color: #f85149;
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>