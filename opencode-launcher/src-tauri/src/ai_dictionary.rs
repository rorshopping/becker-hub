use once_cell::sync::Lazy;
use regex::Regex;

pub const INITIAL_PROMPT: &str = "Transcription uses modern AI developer vocabulary: \
Claude, Anthropic, opencode, Codex, GPT-4, GPT-4o, o1, o3, Gemini, Llama, Mistral, \
Sonnet, Haiku, Opus, Cursor, Aider, Devin, LangChain, RAG, embeddings, tokenizer, \
vector database, Pinecone, Ollama, vLLM, Hugging Face, Mixtral, Qwen, DeepSeek, \
prompt, system prompt, tool call, function calling, agent, MCP, Tauri, Rust, TypeScript, \
React, Svelte, Vite, Docker, Kubernetes, Git, GitHub, VSCode, Neovim, PyTorch, CUDA.";

static CORRECTIONS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    let entries: &[(&str, &str)] = &[
        (r"(?i)\bclouds?\b(?=.*(ai|code|model|anthropic|prompt))", "Claude"),
        (r"(?i)\bclawed\b", "Claude"),
        (r"(?i)\bclode\b", "Claude"),
        (r"(?i)\bcloud\s+(sonnet|haiku|opus)\b", "Claude $1"),
        (r"(?i)\bsonnet\b", "Sonnet"),
        (r"(?i)\bhaiku\b", "Haiku"),
        (r"(?i)\bopus\b", "Opus"),
        (r"(?i)\bg\s?p\s?t\s?-?\s?4\s?o\b", "GPT-4o"),
        (r"(?i)\bgbt\b", "GPT"),
        (r"(?i)\bgpt\s+four\b", "GPT-4"),
        (r"(?i)\bo\s*one\b", "o1"),
        (r"(?i)\bo\s*three\b", "o3"),
        (r"(?i)\bgemini\b", "Gemini"),
        (r"(?i)\bllama\b", "Llama"),
        (r"(?i)\bmixtral\b", "Mixtral"),
        (r"(?i)\bdeep\s*seek\b", "DeepSeek"),
        (r"(?i)\bquen\b|\bkwen\b", "Qwen"),
        (r"(?i)\bopen\s*code\b", "opencode"),
        (r"(?i)\bcodex\b", "Codex"),
        (r"(?i)\bcursor\b", "Cursor"),
        (r"(?i)\baider\b", "Aider"),
        (r"(?i)\bdevin\b", "Devin"),
        (r"(?i)\bol[ao]ma\b", "Ollama"),
        (r"(?i)\bv\s*llm\b", "vLLM"),
        (r"(?i)\blang\s*chain\b", "LangChain"),
        (r"(?i)\bhugging\s*face\b", "Hugging Face"),
        (r"(?i)\brag\b", "RAG"),
        (r"(?i)\bm\s*c\s*p\b", "MCP"),
        (r"(?i)\bl\s*l\s*m\b", "LLM"),
        (r"(?i)\bapi\b", "API"),
        (r"(?i)\bcli\b", "CLI"),
        (r"(?i)\bides?\b", "IDE"),
        (r"(?i)\btowery\b|\btoe-ree\b", "Tauri"),
        (r"(?i)\brust\b", "Rust"),
        (r"(?i)\btype\s*script\b", "TypeScript"),
        (r"(?i)\bsvelt\b", "Svelte"),
        (r"(?i)\bvite\b", "Vite"),
        (r"(?i)\bneo\s*vim\b", "Neovim"),
        (r"(?i)\bv\s*s\s*code\b", "VSCode"),
        (r"(?i)\bpie\s*torch\b", "PyTorch"),
        (r"(?i)\bcooda\b", "CUDA"),
    ];
    entries.iter().map(|(p, r)| (Regex::new(p).unwrap(), *r)).collect()
});

pub fn correct(input: &str) -> String {
    let mut s = input.to_string();
    for (re, rep) in CORRECTIONS.iter() {
        s = re.replace_all(&s, *rep).to_string();
    }
    s
}
