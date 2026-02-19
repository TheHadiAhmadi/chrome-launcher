<script lang="ts">
  import './style.css'
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type Profile = {
    folder: string;
    name: string;
    email?: string;
  };

  type Tag = {
    name: string;
    color: string;
  };

  let profiles: Profile[] = [];
  let search = "";
  let tags: Record<string, Tag[]> = {};
  let selected = 0;
  let isDark = true;

  // Modals state
  let addModalOpen = false;
  let addModalFolder = "";
  let newTagName = "";
  let newTagColor = "#3b82f6"; // Default blue
  
  let removeModalOpen = false;
  let removeModalFolder = "";
  let removeModalTag: Tag | null = null;

  // Premium, modern color palette for tags
  const PRESET_COLORS = [
    "#3b82f6", // Blue
    "#10b981", // Emerald
    "#f59e0b", // Amber
    "#ef4444", // Red
    "#8b5cf6", // Violet
    "#ec4899", // Pink
    "#06b6d4", // Cyan
    "#64748b"  // Slate
  ];

  let selectedFilterTags: string[] = [];
  let tagDropdownOpen = false;

  // Derive unique tags based on name to show in the dropdown
  $: allUniqueTags = Array.from(
    new Map(Object.values(tags).flat().map(t => [t.name, t])).values()
  ).sort((a, b) => a.name.localeCompare(b.name));

  $: filteredProfiles = profiles.filter((p) => {
    const q = search.toLowerCase();
    const profileTagNames = (tags[p.folder] || []).map(t => t.name);
    
    const matchesText =
      p.name.toLowerCase().includes(q) ||
      (p.email || "").toLowerCase().includes(q);
      
    const matchesTags = 
      selectedFilterTags.length === 0 || 
      selectedFilterTags.every((t) => profileTagNames.includes(t));

    return matchesText && matchesTags;
  });

  $: {
    search;
    selectedFilterTags;
    selected = 0;
  }

  onMount(async () => {
    const savedTheme = localStorage.getItem('theme');
    isDark = savedTheme === 'dark' || (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches);
    updateThemeClass();

    try {
      profiles = await invoke("get_profiles");
      const loadedTags: any = await invoke("get_tags");
      
      const parsedTags: Record<string, Tag[]> = {};
      for (const [folder, folderTags] of Object.entries(loadedTags || {})) {
        parsedTags[folder] = (folderTags as any[]).map(t => 
          typeof t === 'string' ? { name: t, color: PRESET_COLORS[0] } : t
        );
      }
      tags = parsedTags;
    } catch (e) {
      console.warn("Using mock/fallback mode. Could not load data from Tauri.");
    }
    
    document.getElementById("search")?.focus();
  });

  function toggleTheme() {
    isDark = !isDark;
    localStorage.setItem('theme', isDark ? 'dark' : 'light');
    updateThemeClass();
  }

  function updateThemeClass() {
    if (isDark) document.documentElement.classList.add('dark');
    else document.documentElement.classList.remove('dark');
  }

  // Improved YIQ contrast for better legibility on mid-tone colors
  function getContrastYIQ(hexcolor: string) {
    hexcolor = hexcolor.replace("#", "");
    if (hexcolor.length === 3) hexcolor = hexcolor.split('').map(c => c+c).join('');
    const r = parseInt(hexcolor.substring(0, 2), 16);
    const g = parseInt(hexcolor.substring(2, 4), 16);
    const b = parseInt(hexcolor.substring(4, 6), 16);
    const yiq = ((r * 299) + (g * 587) + (b * 114)) / 1000;
    return yiq >= 128 ? '#171717' : '#ffffff'; 
  }

  function toggleFilterTag(tagName: string) {
    if (selectedFilterTags.includes(tagName)) {
      selectedFilterTags = selectedFilterTags.filter((t) => t !== tagName);
    } else {
      selectedFilterTags = [...selectedFilterTags, tagName];
    }
    document.getElementById("search")?.focus();
  }

  // --- Modal Logic ---
  function openAddTagModal(folder: string) {
    addModalFolder = folder;
    newTagName = "";
    newTagColor = PRESET_COLORS[Math.floor(Math.random() * PRESET_COLORS.length)];
    addModalOpen = true;
    setTimeout(() => document.getElementById("new-tag-input")?.focus(), 50);
  }

  async function confirmAddTag() {
    if (!newTagName.trim()) {
      addModalOpen = false;
      return;
    }
    const cleanName = newTagName.trim();
    
    if (!(tags[addModalFolder] || []).some(t => t.name.toLowerCase() === cleanName.toLowerCase())) {
      tags[addModalFolder] = [...(tags[addModalFolder] || []), { name: cleanName, color: newTagColor }];
      tags = { ...tags };
      await invoke("save_tags", { tags }).catch(() => {});
    }
    addModalOpen = false;
    document.getElementById("search")?.focus();
  }

  function openRemoveTagModal(folder: string, tag: Tag) {
    removeModalFolder = folder;
    removeModalTag = tag;
    removeModalOpen = true;
  }

  async function confirmRemoveTag() {
    if (removeModalTag && removeModalFolder) {
      tags[removeModalFolder] = (tags[removeModalFolder] || []).filter((t) => t.name !== removeModalTag!.name);
      tags = { ...tags };
      
      const remainingGlobal = Object.values(tags).flat().some(t => t.name === removeModalTag!.name);
      if (!remainingGlobal) {
        selectedFilterTags = selectedFilterTags.filter(x => x !== removeModalTag!.name);
      }
      
      await invoke("save_tags", { tags }).catch(() => {});
    }
    removeModalOpen = false;
    document.getElementById("search")?.focus();
  }

  // --- App Logic ---
  function openProfile(folder: string) {
    invoke("launch_profile", { folder }).catch(() => console.log('Mock launch:', folder));
  }

  function handleKey(e: KeyboardEvent) {
    if (addModalOpen) {
      if (e.key === "Enter") confirmAddTag();
      if (e.key === "Escape") addModalOpen = false;
      return;
    }
    if (removeModalOpen) {
      if (e.key === "Enter") confirmRemoveTag();
      if (e.key === "Escape") removeModalOpen = false;
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selected = Math.min(selected + 1, filteredProfiles.length - 1);
      scrollToSelected();
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      selected = Math.max(selected - 1, 0);
      scrollToSelected();
    }
    if (e.key === "Enter" && filteredProfiles[selected]) {
      openProfile(filteredProfiles[selected].folder);
    }
    if (e.key === "Escape") tagDropdownOpen = false;
  }
  
  function scrollToSelected() {
    setTimeout(() => {
      const el = document.querySelector('.card-selected');
      // Scroll to center for a much smoother visual experience
      if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }, 0);
  }
  
  function handleOutsideClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.tag-filter-container')) tagDropdownOpen = false;
  }
</script>

<svelte:window on:click={handleOutsideClick} />

<div class="min-h-screen bg-neutral-50 dark:bg-neutral-950 text-neutral-900 dark:text-neutral-50 transition-colors duration-500 ease-in-out font-sans selection:bg-blue-500/30">
  
  <main on:keydown={handleKey} tabindex="-1" class="w-full max-w-4xl mx-auto h-screen flex flex-col pt-12 pb-4 px-6 outline-none">
    
    <header class="flex flex-col gap-6 mb-8">
      <div class="flex justify-between items-center">
        <div class="flex items-center gap-3.5">
          <div class="flex items-center justify-center w-11 h-11 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl shadow-sm dark:shadow-[inset_0_1px_0_0_rgba(255,255,255,0.05)] transition-colors duration-500">
            <svg viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="2.5" fill="none" class="text-blue-500 dark:text-blue-400 dark:drop-shadow-[0_0_8px_rgba(96,165,250,0.4)]"><circle cx="12" cy="12" r="10"></circle><circle cx="12" cy="12" r="4"></circle><line x1="21.17" y1="8" x2="12" y2="8"></line><line x1="3.95" y1="6.06" x2="8.54" y2="14"></line><line x1="10.88" y1="21.94" x2="15.46" y2="14"></line></svg>
          </div>
          <h1 class="m-0 text-2xl font-extrabold tracking-tight bg-gradient-to-r from-neutral-900 to-neutral-600 dark:from-white dark:to-neutral-400 bg-clip-text text-transparent">
            Chrome Launcher
          </h1>
        </div>

        <button 
          on:click={toggleTheme} 
          class="p-2.5 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 text-neutral-500 dark:text-neutral-400 hover:text-blue-500 dark:hover:text-blue-400 shadow-sm transition-all duration-300 hover:shadow hover:-translate-y-0.5 focus:outline-none focus:ring-2 focus:ring-blue-500/50 active:scale-95"
          title="Toggle theme"
          aria-label="Toggle theme"
        >
          {#if isDark}
            <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" class="transition-transform duration-500 hover:rotate-12"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg>
          {:else}
            <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" class="transition-transform duration-500 hover:-rotate-12"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>
          {/if}
        </button>
      </div>
      
      <div class="flex flex-col sm:flex-row gap-3 relative z-20">
        <div class="relative flex-1 group">
          <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" class="absolute left-4 top-1/2 -translate-y-1/2 text-neutral-400 dark:text-neutral-500 transition-colors duration-300 group-focus-within:text-blue-500"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
          <input
            id="search"
            class="w-full py-3 pl-11 pr-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white/80 dark:bg-neutral-900/80 backdrop-blur-sm text-neutral-900 dark:text-neutral-100 text-sm transition-all duration-300 shadow-sm focus:outline-none focus:border-blue-500 focus:bg-white dark:focus:bg-neutral-900 focus:ring-4 focus:ring-blue-500/20"
            placeholder="Search profiles or emails..."
            bind:value={search}
            autocomplete="off"
          />
        </div>

        <div class="tag-filter-container relative">
          <button 
            class="flex items-center justify-center gap-2 w-full sm:w-auto h-full px-5 py-3 sm:py-0 rounded-xl text-sm font-semibold transition-all duration-300 border shadow-sm outline-none focus:ring-4 focus:ring-blue-500/20 active:scale-95
            {selectedFilterTags.length > 0 
              ? 'bg-blue-50 dark:bg-blue-500/10 border-blue-500 text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-500/20' 
              : 'bg-white dark:bg-neutral-900/80 border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-50 dark:hover:bg-neutral-800'}" 
            on:click={() => tagDropdownOpen = !tagDropdownOpen}
          >
            <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2.5" fill="none"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon></svg>
            Filter Tags {selectedFilterTags.length > 0 ? `(${selectedFilterTags.length})` : ''}
          </button>

          {#if tagDropdownOpen}
            <div class="absolute top-[calc(100%+8px)] right-0 w-full sm:w-64 bg-white/95 dark:bg-neutral-900/95 backdrop-blur-xl border border-neutral-200 dark:border-neutral-800 rounded-2xl p-2.5 shadow-2xl max-h-72 overflow-y-auto custom-scrollbar animate-slide-down origin-top">
              {#if allUniqueTags.length === 0}
                <div class="p-4 text-center text-neutral-500 text-sm font-medium">No tags available</div>
              {:else}
                <div class="flex flex-col gap-1">
                  {#each allUniqueTags as tag}
                    <label class="flex items-center gap-3 p-2.5 rounded-xl cursor-pointer text-sm text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors">
                      <input 
                        type="checkbox" 
                        class="accent-blue-500 w-4 h-4 rounded border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 cursor-pointer transition-all"
                        checked={selectedFilterTags.includes(tag.name)}
                        on:change={() => toggleFilterTag(tag.name)}
                      />
                      <span class="inline-flex items-center gap-2 font-semibold tracking-wide text-neutral-800 dark:text-neutral-200">
                        <span class="w-3 h-3 rounded-full shadow-inner" style="background-color: {tag.color}"></span>
                        {tag.name}
                      </span>
                    </label>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </header>

    <section class="flex flex-col gap-3 flex-1 overflow-y-auto pr-2 pt-1 pb-6 custom-scrollbar">
      {#if filteredProfiles.length === 0}
        <div class="flex flex-col items-center justify-center h-48 text-neutral-500 text-sm border-2 border-dashed border-neutral-200 dark:border-neutral-800 rounded-3xl bg-white/50 dark:bg-neutral-900/30">
          <svg viewBox="0 0 24 24" width="40" height="40" stroke="currentColor" stroke-width="1.5" fill="none" class="mb-4 opacity-50"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
          <span class="font-medium text-base">No profiles found</span>
          <span class="text-xs opacity-70 mt-1">Try adjusting your search or filters</span>
        </div>
      {/if}

      {#each filteredProfiles as profile, i}
        <div 
          class="flex flex-col sm:flex-row justify-between items-start sm:items-center p-4 sm:px-5 sm:py-4 rounded-2xl border transition-all duration-300 group
          {i === selected 
            ? 'card-selected border-blue-500 bg-blue-50/50 dark:bg-blue-900/10 shadow-md dark:shadow-[0_4px_24px_-8px_rgba(59,130,246,0.3)] ring-1 ring-blue-500 dark:ring-blue-500/50' 
            : 'border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900/60 hover:shadow-md dark:hover:bg-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700'}"
          on:mouseenter={() => selected = i}
          role="button"
          tabindex="-1"
          on:click={() => openProfile(profile.folder)}
        >
          <div class="flex items-start gap-4 w-full sm:w-auto flex-1">
            <div class="w-12 h-12 shrink-0 rounded-full bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center font-bold text-lg text-white shadow-inner ring-2 ring-white/10 dark:ring-white/5">
              {profile.name.charAt(0).toUpperCase()}
            </div>
            
            <div class="flex flex-col w-full">
              <div class="font-bold text-base text-neutral-900 dark:text-neutral-100 tracking-tight">{profile.name}</div>
              {#if profile.email}
                <div class="text-sm font-medium text-neutral-500 dark:text-neutral-400 mt-0.5">{profile.email}</div>
              {/if}

              {#if tags[profile.folder] && tags[profile.folder].length > 0}
                <div class="mt-3 flex flex-wrap gap-2">
                  {#each tags[profile.folder] as tag}
                    <span 
                      class="inline-flex items-center gap-1.5 px-3 py-1 text-[11px] font-bold tracking-wide uppercase rounded-full shadow-sm transition-transform hover:scale-105 cursor-default"
                      style="background-color: {tag.color}; color: {getContrastYIQ(tag.color)};"
                      on:click|stopPropagation
                    >
                      {tag.name}
                      <button 
                        class="ml-0.5 opacity-60 hover:opacity-100 hover:bg-black/20 p-0.5 rounded-full outline-none transition-all focus:bg-black/20" 
                        on:click|stopPropagation={() => openRemoveTagModal(profile.folder, tag)} 
                        aria-label="Remove tag {tag.name}"
                        title="Remove tag"
                      >
                        <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                      </button>
                    </span>
                  {/each}
                </div>
              {/if}
            </div>
          </div>

          <div class="flex items-center gap-2.5 mt-4 sm:mt-0 w-full sm:w-auto justify-end shrink-0 pl-16 sm:pl-0">
            <button 
              class="flex items-center justify-center gap-1.5 px-4 py-2.5 rounded-xl text-sm font-semibold bg-white dark:bg-transparent border border-neutral-200 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-50 dark:hover:bg-neutral-800 hover:text-neutral-900 dark:hover:text-white transition-all duration-200 outline-none focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-800 shadow-sm dark:shadow-none active:scale-95" 
              on:click|stopPropagation={() => openAddTagModal(profile.folder)}
            >
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2.5" fill="none"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path><line x1="7" y1="7" x2="7.01" y2="7"></line></svg>
              Tag
            </button>
            <button 
              class="flex items-center justify-center gap-1.5 px-5 py-2.5 rounded-xl text-sm font-semibold bg-blue-600 text-white shadow-[0_2px_12px_rgba(37,99,235,0.25)] hover:bg-blue-500 hover:shadow-[0_4px_16px_rgba(37,99,235,0.4)] transition-all duration-200 border-none outline-none focus:ring-4 focus:ring-blue-500/30 active:scale-95" 
              on:click|stopPropagation={() => openProfile(profile.folder)}
            >
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2.5" fill="none"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
              Launch
            </button>
          </div>
        </div>
      {/each}
    </section>

    <footer class="text-center pt-5 pb-2 text-neutral-400 dark:text-neutral-600 text-[10px] font-bold tracking-widest uppercase transition-colors duration-500">
      Asre-Naween Computer Services
    </footer>
  </main>
</div>

{#if addModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-neutral-900/50 dark:bg-black/70 backdrop-blur-md animate-in fade-in duration-200" on:click={() => addModalOpen = false}>
    <div class="bg-white dark:bg-neutral-900 w-full max-w-sm rounded-3xl shadow-2xl border border-neutral-200 dark:border-neutral-800 p-7 animate-slide-up" on:click|stopPropagation>
      <h3 class="text-xl font-extrabold text-neutral-900 dark:text-neutral-100 mb-1">Create New Tag</h3>
      <p class="text-sm font-medium text-neutral-500 dark:text-neutral-400 mb-6">Organize this profile with a custom label.</p>
      
      <div class="mb-6">
        <label for="new-tag-input" class="block text-xs font-bold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider mb-2">Tag Name</label>
        <input 
          id="new-tag-input"
          class="w-full py-3 px-4 rounded-xl border border-neutral-300 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-950 text-neutral-900 dark:text-neutral-100 font-medium focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-500/20 transition-all shadow-inner"
          placeholder="e.g., Work, Dev, Personal..."
          bind:value={newTagName}
          on:keydown={(e) => { if (e.key === 'Enter') confirmAddTag(); if (e.key === 'Escape') addModalOpen = false; }}
          autocomplete="off"
        />
      </div>

      <div class="mb-8">
        <label class="block text-xs font-bold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider mb-3">Tag Color</label>
        <div class="flex items-center gap-3 flex-wrap">
          {#each PRESET_COLORS as color}
            <button 
              class="w-8 h-8 rounded-full border-2 transition-transform hover:scale-110 shadow-sm {newTagColor === color ? 'border-white dark:border-neutral-900 ring-2 ring-blue-500 scale-110' : 'border-transparent'}"
              style="background-color: {color};"
              on:click={() => newTagColor = color}
              aria-label="Select color {color}"
            ></button>
          {/each}
          
          <div class="relative w-8 h-8 rounded-full overflow-hidden border-2 border-transparent hover:scale-110 transition-transform shadow-sm {!PRESET_COLORS.includes(newTagColor) ? 'border-white dark:border-neutral-900 ring-2 ring-blue-500 scale-110' : ''}">
            <div class="absolute inset-0 bg-gradient-to-tr from-pink-500 via-red-500 to-amber-500 pointer-events-none"></div>
            <input type="color" class="absolute inset-[-10px] w-[50px] h-[50px] opacity-0 cursor-pointer" bind:value={newTagColor} />
          </div>
        </div>
      </div>
      
      <div class="flex gap-3 justify-end pt-2 border-t border-neutral-100 dark:border-neutral-800">
        <button class="px-5 py-2.5 rounded-xl text-sm font-bold text-neutral-600 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors active:scale-95" on:click={() => { addModalOpen = false; document.getElementById("search")?.focus(); }}>
          Cancel
        </button>
        <button class="px-5 py-2.5 rounded-xl text-sm font-bold text-white bg-blue-600 hover:bg-blue-500 shadow-md transition-all focus:ring-4 focus:ring-blue-500/30 active:scale-95" on:click={confirmAddTag}>
          Save Tag
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- svelte-ignore a11y_click_events_have_key_events -->
{#if removeModalOpen && removeModalTag}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-neutral-900/50 dark:bg-black/70 backdrop-blur-md animate-in fade-in duration-200" on:click={() => removeModalOpen = false}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="bg-white dark:bg-neutral-900 w-full max-w-sm rounded-3xl shadow-2xl border border-neutral-200 dark:border-neutral-800 p-7 text-center animate-slide-up" on:click|stopPropagation>
      
      <div class="mx-auto flex items-center justify-center w-16 h-16 rounded-full bg-red-100 dark:bg-red-500/10 text-red-600 dark:text-red-500 mb-5 shadow-inner">
        <svg viewBox="0 0 24 24" width="32" height="32" stroke="currentColor" stroke-width="2.5" fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
      </div>

      <h3 class="text-xl font-extrabold text-neutral-900 dark:text-neutral-100 mb-2">Remove Tag?</h3>
      <p class="text-sm font-medium text-neutral-500 dark:text-neutral-400 mb-8 leading-relaxed">
        Are you sure you want to remove the <span class="font-bold px-1.5 py-0.5 rounded text-[11px] uppercase tracking-wide" style="background-color: {removeModalTag.color}; color: {getContrastYIQ(removeModalTag.color)};">{removeModalTag.name}</span> tag from this profile?
      </p>
      
      <div class="flex gap-3 w-full">
        <button class="flex-1 py-3 rounded-xl text-sm font-bold text-neutral-600 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors active:scale-95" on:click={() => { removeModalOpen = false; document.getElementById("search")?.focus(); }}>
          Cancel
        </button>
        <button class="flex-1 py-3 rounded-xl text-sm font-bold text-white bg-red-600 hover:bg-red-500 shadow-md transition-all focus:ring-4 focus:ring-red-500/30 active:scale-95" on:click={confirmRemoveTag}>
          Remove
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Base structural reset to ensure no native blue outlines on main container */
  :global(body) { 
    margin: 0; 
    overflow: hidden; /* Prevent body scroll, everything scrolls inside Svelte */
  }
  
  main {
    -webkit-tap-highlight-color: transparent;
  }

  /* Slick Keyframe Animations */
  .animate-slide-down {
    animation: slideDown 0.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  .animate-slide-up {
    animation: slideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-12px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }
  @keyframes slideUp {
    from { opacity: 0; transform: translateY(20px) scale(0.95); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  /* Custom Premium Scrollbar */
  .custom-scrollbar::-webkit-scrollbar { width: 6px; }
  .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
  .custom-scrollbar::-webkit-scrollbar-thumb { 
    background-color: #cbd5e1; /* slate-300 */
    border-radius: 10px; 
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover { background-color: #94a3b8; }
  
  :global(.dark) .custom-scrollbar::-webkit-scrollbar-thumb { background-color: #334155; }
  :global(.dark) .custom-scrollbar::-webkit-scrollbar-thumb:hover { background-color: #475569; }
</style>