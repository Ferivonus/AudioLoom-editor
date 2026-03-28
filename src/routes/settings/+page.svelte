<script lang="ts">
  import { playerState } from '$lib/stores/settings.svelte'; 
  import { invoke } from '@tauri-apps/api/core'; 
  import { open } from '@tauri-apps/plugin-dialog';
  import { fade, fly } from 'svelte/transition';

  const temalar = [
    { id: 'theme-modern', ad: 'Modern Dark', desc: 'Profesyonel ve dengeli', reklam: 'VARSAYILAN' },
    { id: 'theme-orkun-favori', ad: 'Orkun Favori', desc: 'Neon ve yüksek enerji', reklam: 'ORKUN FAVORİ' },
    { id: 'theme-lofi', ad: 'Lo-Fi Night', desc: 'Sakin çalışma modu' },
    { id: 'theme-ghibli', ad: 'Studio Ghibli', desc: 'The Wind Rises estetiği', reklam: 'DEV CHOICE' },
    { id: 'theme-retro', ad: 'Retro 80s', desc: 'Nostaljik arcade' },
    { id: 'theme-ocean', ad: 'Deep Ocean', desc: 'Derin ve huzurlu', reklam: 'EN POPÜLER' },
    { id: 'theme-sakura', ad: 'Sakura Zen', desc: 'Zarif dokunuşlar' },
    { id: 'theme-oled', ad: 'OLED Eclipse', desc: 'Maksimum kontrast' },
    { id: 'theme-light', ad: 'Light Mode', desc: 'Ferah ve aydınlık' },
    { id: 'theme-nord', ad: 'Nordic Dark', desc: 'Yazılımcı dostu soğuk' },
    { id: 'theme-cyberpunk', ad: 'Neon Cyberpunk', desc: 'Yüksek kontrast neon' }
  ];

  const audioFormats = [
    { id: 'wav', name: 'WAV', desc: 'Kayıpsız Yüksek Kalite', color: 'text-primary' },
    { id: 'mp3', name: 'MP3', desc: 'Standart Düşük Boyut', color: 'text-primary-sec' },
    { id: 'flac', name: 'FLAC', desc: 'Sıkıştırılmış Kayıpsız', color: 'text-text-main' },
    { id: 'ogg', name: 'OGG', desc: 'Web Optimize', color: 'text-text-muted' },
    { id: 'aac', name: 'AAC', desc: 'Gelişmiş Ses Bloğu', color: 'text-primary' }
  ];

  const dateFormats = [
    { id: 'DD-MM-YYYY', label: 'Gün-Ay-Yıl', preview: '25-03-2026' },
    { id: 'MM-DD-YYYY', label: 'Ay-Gün-Yıl', preview: '03-25-2026' },
    { id: 'YYYY-MM-DD', label: 'Yıl-Ay-Gün', preview: '2026-03-25' }
  ];

  async function ayarlariKaydet() {
    try {
      const currentSettings = $state.snapshot(playerState);
      await invoke('ayarlari_kaydet', {
        ayarlar: currentSettings
      });
    } catch (e) {
      console.error("Kayıt başarısız:", e);
    }
  }

  async function temaSec(temaId: string) {
    playerState.current_theme = temaId;
    await ayarlariKaydet();
  }

  async function pickDirectory() {
    try {
      const selected = await open({ 
        directory: true, 
        multiple: false, 
        title: 'Ana Kayıt Klasörünü Seçin' 
      });
      if (selected && typeof selected === 'string') {
        playerState.base_export_dir = selected;
        await ayarlariKaydet();
      }
    } catch (err) {
      console.error("Diyalog hatası:", err);
    }
  }

  let previewPath = $derived((() => {
    const sep = playerState.base_export_dir.includes('\\') ? '\\' : '/';
    let base = playerState.base_export_dir || '[SEÇİLMEDİ]';
    let dateStr = '25-03-2026';
    if (playerState.folder_format === 'MM-DD-YYYY') dateStr = '03-25-2026';
    else if (playerState.folder_format === 'YYYY-MM-DD') dateStr = '2026-03-25';
    const folderName = playerState.custom_prefix ? `${playerState.custom_prefix}_${dateStr}` : dateStr;
    const subFolder = playerState.sub_folder ? `${sep}${playerState.sub_folder}` : '';
    return `${base}${sep}${folderName}${subFolder}${sep}Miks_Dosyasi.${playerState.export_format}`;
  })());

</script>

<svelte:head>
  <title>Ayarlar - AudioLoom</title>
</svelte:head>

<div class="p-8 lg:p-12 w-full min-h-full pb-32 flex flex-col relative max-w-6xl mx-auto bg-transparent text-text-main transition-colors duration-500 overflow-y-auto custom-scrollbar">
  
  <button type="button" onclick={() => history.back()} class="flex items-center gap-2 text-text-muted hover:text-primary transition-colors mb-8 group w-fit shrink-0" in:fly={{ x: -20, duration: 500 }}>
    <svg class="w-5 h-5 group-hover:-translate-x-1 transition-transform" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24"><path d="M10 19l-7-7m0 0l7-7m-7 7h18"></path></svg>
    <span class="text-[10px] font-black uppercase tracking-[0.2em]">Ana Sisteme Dön</span>
  </button>

  <header class="mb-12 shrink-0" in:fly={{ y: -20, duration: 600, delay: 100 }}>
    <div class="flex items-center gap-3 mb-4">
      <span class="flex h-2.5 w-2.5 relative">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
        <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-primary"></span>
      </span>
      <span class="text-[10px] font-black uppercase tracking-[0.4em] text-primary">System Configuration</span>
    </div>
    <h1 class="text-5xl lg:text-7xl font-black tracking-tighter uppercase italic leading-none truncate drop-shadow-lg">Ayarlar</h1>
    <p class="text-text-muted mt-4 font-bold text-sm uppercase tracking-widest opacity-60">AudioLoom Terminal v4.0</p>
  </header>

  <div class="grid gap-12">
    <section class="space-y-8" in:fade={{ delay: 200 }}>
      <div class="flex items-end justify-between border-b border-border pb-4">
        <div>
          <h2 class="text-2xl font-black uppercase italic tracking-tight text-text-main">Görünüm</h2>
          <p class="text-text-muted text-xs font-bold uppercase tracking-widest mt-1">Sistem arayüzü ve renk paletleri</p>
        </div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {#each temalar as tema}
          <button 
            type="button"
            onclick={() => temaSec(tema.id)}
            class="flex flex-col gap-4 group text-left relative transition-all duration-300 hover:-translate-y-1 active:scale-95 w-full min-w-0 {tema.id}"
          >
            <div 
              class="aspect-16/10 w-full rounded-2xl border-2 transition-all duration-500 relative overflow-hidden p-3 flex flex-col gap-2 shrink-0 bg-background text-text-main
              {playerState.current_theme === tema.id 
                ? 'border-primary shadow-[0_15px_40px_rgba(0,0,0,0.4),0_0_20px_var(--accent)] ring-1 ring-primary' 
                : 'border-border opacity-80 hover:opacity-100 hover:border-border-hover hover:shadow-xl'}"
            >
              {#if tema.reklam}
                <div class="absolute top-0 right-0 z-10 bg-primary px-3 py-1 rounded-bl-xl shadow-lg">
                  <span class="text-[8px] font-black uppercase tracking-[0.2em] text-background">{tema.reklam}</span>
                </div>
              {/if}

              <div class="w-full h-4 bg-surface rounded border border-border flex items-center px-2 gap-1.5 shrink-0">
                <div class="w-1.5 h-1.5 rounded-full bg-primary opacity-80"></div>
                <div class="w-1.5 h-1.5 rounded-full bg-primary-sec opacity-80"></div>
                <div class="w-1.5 h-1.5 rounded-full bg-text-muted opacity-30"></div>
              </div>

              <div class="flex flex-1 gap-2 min-h-0">
                <div class="w-1/4 h-full bg-surface rounded border border-border flex flex-col gap-1.5 p-1.5">
                  <div class="w-full h-1 bg-primary rounded-full opacity-80"></div>
                  <div class="w-3/4 h-1 bg-text-muted rounded-full opacity-40"></div>
                  <div class="w-5/6 h-1 bg-text-muted rounded-full opacity-40"></div>
                </div>
                
                <div class="flex-1 h-full bg-surface rounded border border-border flex items-center justify-center p-2 relative overflow-hidden">
                  <div class="flex items-center gap-1 h-full w-full justify-center">
                    <div class="w-1 h-1/3 bg-primary rounded-full"></div>
                    <div class="w-1 h-2/3 bg-primary-sec rounded-full"></div>
                    <div class="w-1 h-full bg-primary rounded-full shadow-[0_0_8px_var(--accent)]"></div>
                    <div class="w-1 h-1/2 bg-primary-sec rounded-full"></div>
                    <div class="w-1 h-1/4 bg-primary rounded-full"></div>
                  </div>
                </div>
              </div>

              {#if playerState.current_theme === tema.id}
                <div class="absolute inset-0 flex items-center justify-center backdrop-blur-[1px] z-30 bg-primary/5" in:fade>
                  <div class="w-10 h-10 bg-primary text-background rounded-full flex items-center justify-center shadow-[0_0_20px_var(--accent)] scale-100 shrink-0">
                    <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
                  </div>
                </div>
              {/if}
            </div>

            <div class="px-1 min-w-0">
              <span class="text-sm font-black uppercase tracking-widest block transition-colors truncate {playerState.current_theme === tema.id ? 'text-primary' : 'group-hover:text-primary'}">{tema.ad}</span>
              <span class="text-[10px] text-text-muted font-bold uppercase tracking-tighter opacity-70 truncate block">{tema.desc}</span>
            </div>
          </button>
        {/each}
        </div>
    </section>

    <section class="bg-surface border border-border rounded-3xl p-8 lg:p-10 shadow-2xl relative overflow-hidden" in:fade={{ delay: 300 }}>
      <div class="flex items-center gap-5 mb-10">
        <div class="p-4 bg-primary/10 rounded-2xl text-primary shadow-inner">
          <svg class="w-7 h-7" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24"><path d="M20 7H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2z"></path><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path></svg>
        </div>
        <div>
          <h2 class="text-2xl font-black uppercase italic tracking-tight text-text-main">Dışa Aktarma Protokolü</h2>
          <p class="text-text-muted text-[10px] uppercase tracking-widest font-bold mt-1">Sistem formatı ve dizin yapılandırması</p>
        </div>
      </div>

      <div class="space-y-10">
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {#each audioFormats as format}
            <button 
              type="button" 
              onclick={async () => { playerState.export_format = format.id; await ayarlariKaydet(); }} 
              class="group relative flex flex-col p-6 bg-background border border-border rounded-2xl transition-all hover:border-border-hover {playerState.export_format === format.id ? 'ring-2 ring-primary border-transparent shadow-[0_0_15px_var(--accent)]' : ''}"
            >
              <span class="text-3xl font-black uppercase tracking-tighter italic transition-colors {format.color} {playerState.export_format === format.id ? '' : 'opacity-40 group-hover:opacity-100'}">{format.id}</span>
              <span class="text-[10px] font-black uppercase tracking-widest mt-1 text-text-muted">{format.desc}</span>
              {#if playerState.export_format === format.id}
                <div class="absolute top-5 right-5 w-2 h-2 bg-primary rounded-full animate-pulse shadow-[0_0_10px_var(--accent)]"></div>
              {/if}
            </button>
          {/each}
        </div>

        <div class="space-y-8">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="space-y-2">
              <label for="customPrefix" class="text-[10px] font-black text-text-muted uppercase tracking-widest ml-1 opacity-60">Kök İsim</label>
              <input 
                id="customPrefix" 
                type="text" 
                bind:value={playerState.custom_prefix} 
                onblur={ayarlariKaydet} 
                class="w-full bg-background border border-border rounded-xl p-4 text-sm outline-none focus:border-primary transition-all font-bold text-text-main placeholder:opacity-20" 
              />
            </div>
            
            <div class="space-y-2">
              <label for="folderFormat" class="text-[10px] font-black text-text-muted uppercase tracking-widest ml-1 opacity-60">Zaman Dizilimi</label>
              <select 
                id="folderFormat" 
                bind:value={playerState.folder_format} 
                onchange={ayarlariKaydet} 
                class="w-full bg-background border border-border rounded-xl p-4 text-sm outline-none focus:border-primary transition-all font-bold text-text-main appearance-none cursor-pointer"
              >
                {#each dateFormats as df}<option value={df.id}>{df.label} ({df.preview})</option>{/each}
              </select>
            </div>
            
            <div class="space-y-2">
              <label for="subFolder" class="text-[10px] font-black text-text-muted uppercase tracking-widest ml-1 opacity-60">Alt Dizin</label>
              <input 
                id="subFolder" 
                type="text" 
                bind:value={playerState.sub_folder} 
                onblur={ayarlariKaydet} 
                class="w-full bg-background border border-border rounded-xl p-4 text-sm outline-none focus:border-primary transition-all font-bold text-text-main placeholder:opacity-20" 
              />
            </div>
          </div>

          <div class="space-y-4 pt-4 border-t border-border/10">
            <p class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] ml-1">Nihai Kayıt Yolu</p>
            <div class="flex flex-col md:flex-row items-center gap-4 bg-background p-3 rounded-2xl border border-border group hover:border-border-hover transition-colors">
              <code class="text-primary text-xs truncate flex-1 font-mono px-4 select-all bg-black/20 py-4 rounded-xl block w-full tracking-tighter opacity-80 uppercase">{playerState.base_export_dir ? previewPath : 'SİSTEM DİZİNİ SEÇİLMEDİ'}</code>
              <button type="button" onclick={pickDirectory} class="w-full md:w-auto bg-text-main text-background hover:bg-primary hover:text-background text-[10px] font-black py-4 px-10 rounded-xl transition-all active:scale-95 uppercase tracking-[0.2em] shrink-0 shadow-lg">Protokol Dizinini Değiştir</button>
            </div>
          </div>
        </div>

        <button 
          type="button" 
          onclick={async () => { playerState.ask_every_time = !playerState.ask_every_time; await ayarlariKaydet(); }} 
          class="flex items-center justify-between p-6 bg-background rounded-2xl border border-border hover:border-border-hover transition-all group w-full"
        >
          <div class="text-left">
            <p class="text-xs font-black uppercase tracking-widest text-text-main">Hızlı Otomatik Kayıt</p>
            <p class="text-[10px] text-text-muted mt-1 font-bold uppercase opacity-60">Her işlemde onay penceresini atla</p>
          </div>
          <div class="w-12 h-6 rounded-full relative transition-colors shrink-0 {!playerState.ask_every_time ? 'bg-primary shadow-[0_0_10px_var(--accent)]' : 'bg-surface border border-border'}">
            <div class="absolute top-1 w-4 h-4 bg-white rounded-full transition-all {!playerState.ask_every_time ? 'left-7' : 'left-1 bg-text-muted'}"></div>
          </div>
        </button>
      </div>
    </section>

    <footer class="flex flex-col items-center justify-center py-20 group shrink-0" in:fade={{ delay: 500 }}>
      <div class="w-16 h-16 bg-primary rounded-2xl flex items-center justify-center mb-6 rotate-3 group-hover:rotate-0 transition-transform duration-700 shadow-[0_15px_40px_rgba(0,0,0,0.4),0_0_20px_var(--accent)]">
        <svg class="w-8 h-8 text-background" fill="currentColor" viewBox="0 0 24 24"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
      </div>
      <h3 class="text-3xl font-black tracking-[0.4em] uppercase italic drop-shadow-2xl text-text-main">AudioLoom</h3>
      <div class="mt-8 flex flex-col items-center gap-2">
        <p class="text-[11px] font-black text-primary uppercase tracking-[0.5em] animate-pulse px-4 text-center">Fahrettin Baştürk tarafından yapıldı</p>
        <div class="flex items-center justify-center gap-3 mt-1 opacity-30">
          <span class="h-px w-8 bg-text-muted"></span>
          <p class="text-[9px] font-mono text-text-muted font-bold uppercase tracking-widest">System v4.0 Stable Build</p>
          <span class="h-px w-8 bg-text-muted"></span>
        </div>
      </div>
    </footer>
  </div>
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar { width: 4px; }
  .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
  .custom-scrollbar::-webkit-scrollbar-thumb { background: var(--border); border-radius: 10px; }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: var(--accent); }

  button {
    cursor: pointer;
    outline: none;
  }

  select {
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%2371717a'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 1rem center;
    background-size: 1.25rem;
  }
</style>