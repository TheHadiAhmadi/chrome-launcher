<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type Profile = {
    folder: string;
    name: string;
    email?: string;
  };

  let profiles: Profile[] = [];
  let search = "";
  let tags: Record<string, string[]> = {};
  let selected = 0;

  onMount(async () => {
    profiles = await invoke("get_profiles");
    tags = await invoke("get_tags");
    document.getElementById("search")?.focus();
  });

  function filteredProfiles() {
    const q = search.toLowerCase();

    return profiles.filter((p) => {
      const profileTags = tags[p.folder] || [];
      return (
        p.name.toLowerCase().includes(q) ||
        (p.email || "").toLowerCase().includes(q) ||
        profileTags.some((t) => t.toLowerCase().includes(q))
      );
    });
  }

  async function addTag(folder: string) {
    const value = prompt("Enter tag");
    if (!value) return;
    tags[folder] = [...(tags[folder] || []), value];
    await invoke("save_tags", { tags });
  }

  async function removeTag(folder: string, tag: string) {
    tags[folder] = (tags[folder] || []).filter((t) => t !== tag);
    await invoke("save_tags", { tags });
  }

  function openProfile(folder: string) {
    invoke("launch_profile", { folder });
  }

  function handleKey(e: KeyboardEvent) {
    const list = filteredProfiles();
    if (e.key === "ArrowDown") {
      selected = Math.min(selected + 1, list.length - 1);
    }
    if (e.key === "ArrowUp") {
      selected = Math.max(selected - 1, 0);
    }
    if (e.key === "Enter" && list[selected]) {
      openProfile(list[selected].folder);
    }
  }
</script>

<main on:keydown={handleKey} tabindex="0">
  <header>
    <h1>Chrome Launcher</h1>
    <input
      id="search"
      placeholder="Search name, email, tag..."
      bind:value={search}
    />
  </header>

  <section class="profiles">
    {#each filteredProfiles() as profile, i}
      <div class:selected={i === selected} class="card">
        <div class="info">
          <div class="name">{profile.name}</div>
          <div class="email">{profile.email}</div>

          <div class="tags">
            {#each tags[profile.folder] || [] as tag}
              <span
                class="tag"
                on:click={() => removeTag(profile.folder, tag)}
              >
                {tag}
              </span>
            {/each}
          </div>
        </div>

        <div class="actions">
          <button class="open" on:click={() => openProfile(profile.folder)}>
            Open
          </button>
          <button class="tag-btn" on:click={() => addTag(profile.folder)}>
            Tag
          </button>
        </div>
      </div>
    {/each}
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
  }
  
  main {
    height: 100vh;
    background: #111;
    color: #eee;
    font-family: system-ui, -apple-system, sans-serif;
    padding: 24px;
    box-sizing: border-box;
    outline: none;
  }

  header {
    margin-bottom: 20px;
  }

  h1 {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 12px;
  }

  input {
    width: 100%;
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid #333;
    background: #1c1c1c;
    color: #fff;
    font-size: 14px;
  }

  input:focus {
    border-color: #4c8bf5;
  }

  .profiles {
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
    max-height: calc(100vh - 120px);
  }

  .card {
    display: flex;
    justify-content: space-between;
    background: #1a1a1a;
    padding: 16px;
    border-radius: 10px;
    border: 1px solid #222;
    transition: background 0.15s, border 0.15s;
  }

  .card:hover {
    background: #202020;
  }

  .card.selected {
    border: 1px solid #4c8bf5;
    background: #1e2433;
  }

  .info {
    flex: 1;
  }

  .name {
    font-weight: 600;
    font-size: 15px;
  }

  .email {
    font-size: 12px;
    color: #888;
    margin-top: 4px;
  }

  .tags {
    margin-top: 8px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag {
    background: #2d4ed8;
    padding: 4px 8px;
    font-size: 11px;
    border-radius: 6px;
    cursor: pointer;
  }

  .tag:hover {
    opacity: 0.8;
  }

  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  button {
    border: none;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    cursor: pointer;
  }

  .open {
    background: #16a34a;
    color: white;
  }

  .tag-btn {
    background: #7c3aed;
    color: white;
  }

  button:hover {
    opacity: 0.85;
  }
</style>
