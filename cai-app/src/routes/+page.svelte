<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, emit } from "@tauri-apps/api/event";
  import { Send } from "@lucide/svelte";
  import { onMount } from "svelte";

  type MsgRole = "Assistant" | "System" | "Error" | "User";

  type MsgType =
    | {
        type: "Plain";
        content: string;
      }
    | {
        type: "TitleChildren";
        title: string;
        content: string[];
      };

  // Some test data
  let messages = [
    {
      role: "System" as MsgRole,
      type: {
        type: "Plain",
        content: "System message",
      } as MsgType,
    },

    {
      role: "Assistant" as MsgRole,
      type: {
        type: "TitleChildren",
        title: "Assistant message",
        content: ["Child 1", "Child 2"],
      } as MsgType,
    },
    {
      role: "Error" as MsgRole,
      type: {
        type: "Plain",
        content: "Error message",
      } as MsgType,
    },

    {
      role: "User" as MsgRole,
      type: {
        type: "TitleChildren",
        title: "User message",
        content: ["Child 1", "Child 2"],
      } as MsgType,
    },
  ];


  let input_enabled = true;
  let input_value = "";

  listen("request-user-input", async () => {
    // focus / open your input control
    const value = await wait_until_send(); // your UI logic
    // bounce the value back to Rust
    await invoke("deliver_user_input", { value });
  });

  async function wait_until_send() {
    return new Promise((resolve) => {
      const handleKeyDown = (event: KeyboardEvent) => {
        if (event.key === "Enter") {
          resolve(input_value);
          input_enabled = false;
          window.removeEventListener("keydown", handleKeyDown);
        }
      };
      window.addEventListener("keydown", handleKeyDown);
    });
  }


  onMount(async () => {

    // Listen to server ping to send user input
    await listen("js please send user input")

  })

</script>

<main
  class="bg-black text-white grid grid-cols-1 grid-rows-[5fr_1fr] h-screen w-screen"
>
  <!-- Chat cotainer -->
  <div
    class="flex flex-col
    overflow-y-auto
    p-4 mx-auto
    w-full
    max-w-2lg
    "
  >
    {#each messages as message}
      <!-- Message container -->
      <div class="flex flex-col p-1 m-1 bg-[#19191a] rounded-md">
        <div class="text-md font-semibold bg-[#292956] p-1 rounded-sm w-fit">
          {message.role}
        </div>

        {#if message.type.type === "Plain"}
          <div class="p-2 rounded-md">{message.type.content}</div>
        {:else if message.type.type === "TitleChildren"}
          <div class="p-2 rounded-md">
            <h3 class="text-lg font-semibold">{message.type.title}</h3>

            <ul class=" pl-4">
              {#each message.type.content as child}
                <li>{child}</li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Input container -->
  <div
    class="flex flex-row p-1 m-1 bg-[#0d0d0e] rounded-md
    min-h-[5dvh]
    items-center
    justify-center
    w-auto

    "
  >
    <input
      bind:value={input_value}
      disabled={!input_enabled}
      type="text"
      placeholder="Some high effort prompt..."
      class="
        max-w-md
        p-1
        bg-[#0b0b14]
        rounded-md"
    />
    <button class="p-1 text-white border-2 bg-[inherit] rounded-full">
      <Send />
    </button>
  </div>
</main>
