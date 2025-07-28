CAI

A customizable Rust-powered interface to chat with OpenAI's models from your terminal.

This is not just a chatbot—it’s a framework. Designed for developers who want a fast, scriptable way to interact with AI models. It supports persistent memory, command execution, and NEVER will have a GUI

---

Features

Terminal-based Chat – Communicate directly with OpenAI’s GPT via the console.

Persistent Memory – AI remembers past context between sessions.

Command Execution – Let the AI trigger shell commands securely.

File Writing – AI can write to your local filesystem using code blocks.

Chat History – Automatically saves and loads prior conversations.



---

Installation

1. Clone the Repository

git clone https://github.com/hyvnova/cai

2. Set Your OpenAI API Key

macOS / Linux

export OPENAI_API_KEY=your-key-here

Windows (CMD)

set OPENAI_API_KEY=your-key-here

> You can also permanently add this to your environment variables for convenience.



3. Build and Run the App

cargo run --release

You're in. Type a message, hit enter. Type q to quit.


---

Requirements

Rust (stable toolchain)

OpenAI API key

Works best on Windows (Linux/macOS support available)



---

Roadmap

[x] Console AI chat

[x] Persistent memory

[x] Shell + file command execution

[ ] Web UI (in development)

[ ] Configurable plugins and custom commands



---

License

MIT. Free to use, modify, or destroy your terminal with.


---

> Built for devs who’d rather type than tab



