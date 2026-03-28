<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { playerState, type Settings } from '$lib/stores/settings.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(async () => {
      try {
          const ayarlar = await invoke<Settings>('ayarlari_getir');
          
          playerState.current_theme = ayarlar.current_theme;
          playerState.export_format = ayarlar.export_format;
          playerState.base_export_dir = ayarlar.base_export_dir;
          playerState.folder_format = ayarlar.folder_format;
          playerState.custom_prefix = ayarlar.custom_prefix;
          playerState.sub_folder = ayarlar.sub_folder;
          playerState.ask_every_time = ayarlar.ask_every_time;
      } catch (e) {
          console.error("Tauri ayarları yüklenirken hata oluştu:", e);
      }
  });

  $effect(() => {
    if (typeof document !== 'undefined') {
      const body = document.body;
      const currentThemes = Array.from(body.classList).filter(c => c.startsWith('theme-'));
      if (currentThemes.length > 0) body.classList.remove(...currentThemes);
      body.classList.add(playerState.current_theme);
    }
  });
</script>

<div class="{playerState.current_theme} h-screen w-screen flex flex-col bg-background text-text-main font-sans overflow-hidden transition-colors duration-500">
  
  <nav class="h-12 bg-background border-b border-border flex items-center px-6 shrink-0 z-50 shadow-sm transition-colors duration-500">
    <div class="flex items-center gap-8 w-full">
      <div class="flex items-center gap-2 cursor-default select-none group">
         <svg class="w-5 h-5 text-primary transition-transform duration-300 group-hover:rotate-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
           <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"></path>
         </svg>
         <h1 class="text-lg font-semibold tracking-wide text-text-main">Audio<span class="text-primary">Loom</span></h1>
      </div>
      <div class="flex items-center gap-1">
        <a href="/" class="px-3 py-1.5 rounded-md text-sm font-medium border transition-all {$page.url.pathname === '/' ? 'bg-surface text-primary border-border shadow-sm' : 'border-transparent text-text-muted hover:text-text-main hover:bg-surface'}">Editör</a>
        <a href="/settings" class="px-3 py-1.5 rounded-md text-sm font-medium border transition-all {$page.url.pathname.startsWith('/settings') ? 'bg-surface text-primary border-border shadow-sm' : 'border-transparent text-text-muted hover:text-text-main hover:bg-surface'}">Ayarlar</a>
        <a href="/help" class="px-3 py-1.5 rounded-md text-sm font-medium border transition-all {$page.url.pathname.startsWith('/help') ? 'bg-surface text-primary border-border shadow-sm' : 'border-transparent text-text-muted hover:text-text-main hover:bg-surface'}">Yardım</a>
      </div>
    </div>
  </nav>

  <div class="flex-1 flex flex-col relative overflow-hidden bg-background transition-colors duration-500">
    {@render children()}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background-color: var(--bg-main);
  }
</style>