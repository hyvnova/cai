# Console AI Framework

A simple Rust command-line tool for interacting with AI models (like OpenAI's GPT) from your terminal. It supports basic conversation, persistent memory, and lets the AI run shell commands or write files if needed.

---

## Features
- Chat with an AI model in your terminal
- Save and recall information between sessions (memory)
- AI can run shell commands and write files using special blocks
- Keeps a history of your conversations

---

## Getting Started
1. Set your API key:

   Linux/Mac:
   ```sh
   export OPENAI_API_KEY=your-key-here
   ```

   Windows:
   ```sh
   set OPEN
   AI_API_KEY=your-key-here
   ```


2. Run the program:
   ```sh
   cargo run --release
   ```
3. Type your message and press Enter. Type `q` to quit.

---

## Requirements
- Rust (stable)
- OpenAI API key
- Windows (default, but adaptable)

---

## License
MIT
