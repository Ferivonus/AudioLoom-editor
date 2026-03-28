<script lang="ts">
  import WaveSurfer from 'wavesurfer.js';
  import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.esm.js';
  import { onDestroy } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';

  interface Theme {
    bgClass: string;
    borderClass: string;
    waveColorHex: string;
    progressColorHex: string;
  }

  interface Track extends Theme {
    id: number;
    displayName: string;
    volume: number;
    pan: "Left" | "Center" | "Right";
    isMuted: boolean;
    isSolo: boolean;
    audioUrl: string;
    realPath: string; 
  }

  type RegionPluginType = ReturnType<typeof RegionsPlugin.create>;
  type SingleRegionType = ReturnType<RegionPluginType['getRegions']>[0];

  const defaultThemes: Theme[] = [
    { bgClass: "bg-emerald-500/20", borderClass: "border-emerald-500/50", waveColorHex: "#34d399", progressColorHex: "#059669" },
    { bgClass: "bg-amber-500/20", borderClass: "border-amber-500/50", waveColorHex: "#fbbf24", progressColorHex: "#d97706" },
    { bgClass: "bg-purple-500/20", borderClass: "border-purple-500/50", waveColorHex: "#a855f7", progressColorHex: "#7e22ce" },
  ];

  const cutTheme: Theme = { bgClass: "bg-rose-500/20", borderClass: "border-rose-500/50", waveColorHex: "#fb7185", progressColorHex: "#e11d48" };
  const trimTheme: Theme = { bgClass: "bg-cyan-500/20", borderClass: "border-cyan-500/50", waveColorHex: "#22d3ee", progressColorHex: "#0891b2" };

  let tracks: Track[] = [];
  
  const wsInstances = new Map<number, WaveSurfer>();
  const regionsPlugins = new Map<number, RegionPluginType>();

  let isPlaying: boolean = false;
  let isProcessing: boolean = false;
  let globalZoom: number = 20;
  
  let globalScrollX: number = 0;
  let maxDuration: number = 120;
  let timeMarkers: { label: string }[] = [];

  let activeRegion: SingleRegionType | null = null;
  let activeTrackId: number | null = null;

  let regionInputStart: string = "0.00";
  let regionInputEnd: string = "0.00";
  let regionError: string = "";

  let sidebarWidth: number = 256; 
  let isResizing: boolean = false;

  $: {
    timeMarkers = [];
    const count = Math.ceil(maxDuration / 15) + 5;
    for(let i = 0; i < count; i++) {
      const totalSeconds = i * 15;
      const m = Math.floor(totalSeconds / 60).toString().padStart(2, '0');
      const s = (totalSeconds % 60).toString().padStart(2, '0');
      timeMarkers.push({ label: `00:${m}:${s}` });
    }
  }

  function formatPreciseTime(seconds: number): string {
    const m = Math.floor(seconds / 60).toString().padStart(2, '0');
    const s = Math.floor(seconds % 60).toString().padStart(2, '0');
    const ms = Math.floor((seconds % 1) * 100).toString().padStart(2, '0');
    return `${m}:${s}.${ms}`;
  }

  function handleKeydown(e: KeyboardEvent) {
    const activeTag = document.activeElement?.tagName;
    if (e.code === 'Space' && activeTag !== 'INPUT' && activeTag !== 'TEXTAREA') {
      e.preventDefault();
      handlePlayPause();
    }
  }

  function startResize(e: MouseEvent) {
    isResizing = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none'; 
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    sidebarWidth = Math.min(Math.max(e.clientX, 220), 600);
  }

  function stopResize() {
    if (isResizing) {
      isResizing = false;
      document.body.style.cursor = 'default';
      document.body.style.userSelect = 'auto';
    }
  }

  function updateRegionFromInputs() {
    if (!activeRegion || activeTrackId === null) return;
    
    let s = parseFloat(regionInputStart);
    let e = parseFloat(regionInputEnd);

    if (isNaN(s) || isNaN(e)) return;

    if (s >= e) {
      regionError = "Sağ taraf (Bitiş), soldan (Başlangıç) daha küçük veya eşit olamaz!";
      return;
    }

    const ws = wsInstances.get(activeTrackId);
    const duration = ws ? ws.getDuration() : maxDuration;
    
    if (e > duration) e = duration;
    if (s < 0) s = 0;

    regionError = "";
    activeRegion.setOptions({ start: s, end: e });
  }

  function handleResizeKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') {
      sidebarWidth = Math.max(sidebarWidth - 10, 220);
      e.preventDefault();
    } else if (e.key === 'ArrowRight') {
      sidebarWidth = Math.min(sidebarWidth + 10, 600);
      e.preventDefault();
    }
  }

  function moveTrack(index: number, direction: 'up' | 'down') {
    const newTracks = [...tracks];
    if (direction === 'up' && index > 0) {
      [newTracks[index - 1], newTracks[index]] = [newTracks[index], newTracks[index - 1]];
      tracks = newTracks;
    } else if (direction === 'down' && index < tracks.length - 1) {
      [newTracks[index + 1], newTracks[index]] = [newTracks[index], newTracks[index + 1]];
      tracks = newTracks;
    }
  }

  function handlePlayPause() {
    if (wsInstances.size === 0 || isProcessing) return;
    isPlaying = !isPlaying;
    try {
      wsInstances.forEach(ws => isPlaying ? ws.play() : ws.pause());
    } catch (e) {
      isPlaying = false;
    }
  }

  function handleStop() {
    isPlaying = false;
    wsInstances.forEach(ws => {
      ws.pause();
      ws.seekTo(0);
    });
  }

  async function triggerFileInput(): Promise<void> {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Ses Dosyaları', extensions: ['mp3', 'wav', 'flac', 'ogg', 'm4a', 'aac'] }]
      });
      if (!selected) return;
      const filePath = Array.isArray(selected) ? selected[0] : (selected as string);
      const fileName = filePath.split(/[/\\]/).pop() || 'Bilinmeyen Dosya';
      const assetUrl = convertFileSrc(filePath);

      addTrackToUI(fileName, assetUrl, filePath);
    } catch (error) {
      console.error("Hata:", error);
    }
  }

  function addTrackToUI(displayName: string, audioUrl: string, realPath: string, specificTheme?: Theme) {
    const newTrackId = Date.now();
    const theme = specificTheme || defaultThemes[tracks.length % defaultThemes.length];

    tracks = [...tracks, { 
      id: newTrackId, 
      displayName, 
      volume: 80, 
      pan: "Center", 
      isMuted: false, 
      isSolo: false,
      ...theme, 
      audioUrl,
      realPath
    }];
  }

  async function exportSingleTrack(track: Track) {
    try {
      const savePath = await save({
        title: 'Ses Kanalını Dışa Aktar',
        defaultPath: track.displayName.endsWith('.wav') ? track.displayName : `${track.displayName}.wav`,
        filters: [{ name: 'WAV Ses Dosyası', extensions: ['wav'] }]
      });

      if (!savePath) return;
      isProcessing = true;

      await invoke('export_single_track', {
        sourcePath: track.realPath,
        destPath: savePath,
        volume: track.volume / 100,
        pan: track.pan
      });

      alert(`${track.displayName} başarıyla dışa aktarıldı!`);
    } catch (error) {
      console.error("Dışa aktarma hatası:", error);
      alert(`Hata oluştu: ${error}`);
    } finally {
      isProcessing = false;
    }
  }

  async function handleExport() {
    if (tracks.length === 0) return alert("Dışa aktarılacak ses kanalı yok!");

    try {
      const savePath = await save({
        title: 'Projeyi Dışa Aktar',
        defaultPath: 'AudioLoom_Mix.wav',
        filters: [{ name: 'WAV Ses Dosyası', extensions: ['wav'] }]
      });

      if (!savePath) return;

      isProcessing = true;
      handleStop();

      const isAnySolo = tracks.some(t => t.isSolo);
      const activeTracks = tracks.map(track => {
        let finalVolume = track.volume / 100;
        finalVolume = isAnySolo ? ((track.isSolo && !track.isMuted) ? finalVolume : 0) : (track.isMuted ? 0 : finalVolume);
        return { path: track.realPath, volume: finalVolume, pan: track.pan };
      }).filter(t => t.volume > 0);

      if (activeTracks.length === 0) {
        alert("Duyulabilir bir ses kanalı yok (Hepsi sessizde olabilir).");
        isProcessing = false;
        return;
      }

      await invoke('export_project', { tracks: activeTracks, outputPath: savePath });
      alert("Proje başarıyla dışa aktarıldı!");

    } catch (error) {
      alert(`Dışa aktarma sırasında hata: ${error}`);
    } finally {
      isProcessing = false;
    }
  }

  function removeTrack(trackId: number) {
    if (!confirm("Bu kanalı silmek istediğinize emin misiniz?")) return;

    const ws = wsInstances.get(trackId);
    if (ws) {
      ws.destroy();
      wsInstances.delete(trackId);
      regionsPlugins.delete(trackId);
    }
    tracks = tracks.filter(t => t.id !== trackId);
    if (tracks.length === 0) {
      isPlaying = false;
      maxDuration = 120;
    } else {
      maxDuration = 120;
      wsInstances.forEach(instance => {
        const duration = instance.getDuration();
        if (duration > maxDuration) maxDuration = duration;
      });
    }
    clearRegion();
  }

  $: {
    const isAnySolo = tracks.some(t => t.isSolo);
    tracks.forEach(track => {
      const ws = wsInstances.get(track.id);
      if (ws) {
        let finalVolume = track.volume / 100;
        finalVolume = isAnySolo ? ((track.isSolo && !track.isMuted) ? finalVolume : 0) : (track.isMuted ? 0 : finalVolume);
        ws.setVolume(finalVolume);
      }
    });
  }

  $: wsInstances.forEach(ws => ws.zoom(globalZoom));

  function waveformAction(node: HTMLElement, track: Track) {
    const ws = WaveSurfer.create({
      container: node, waveColor: track.waveColorHex, progressColor: track.progressColorHex,
      barWidth: 2, barGap: 2, barRadius: 2, height: 80, url: track.audioUrl,
      cursorColor: '#ffffff', cursorWidth: 2, minPxPerSec: globalZoom,
      hideScrollbar: false, autoScroll: true, autoCenter: true, fetchParams: { mode: 'cors' }
    });

    wsInstances.set(track.id, ws);
    const wsRegions = ws.registerPlugin(RegionsPlugin.create());
    regionsPlugins.set(track.id, wsRegions);
    wsRegions.enableDragSelection({ color: `${track.waveColorHex}40` });

    wsRegions.on('region-created', (region) => {
      regionsPlugins.forEach((plugin, id) => { if (id !== track.id) plugin.clearRegions(); });
      const myRegions = wsRegions.getRegions();
      if(myRegions.length > 1) myRegions[0].remove();
      activeRegion = region; 
      activeTrackId = track.id;
      regionInputStart = region.start.toFixed(2);
      regionInputEnd = region.end.toFixed(2);
      regionError = "";
    });

    wsRegions.on('region-updated', (region) => {
      if (activeRegion && activeRegion.id === region.id) {
        if (document.activeElement?.tagName !== 'INPUT' || (document.activeElement as HTMLInputElement).type !== 'number') {
          regionInputStart = region.start.toFixed(2);
          regionInputEnd = region.end.toFixed(2);
        }
        regionError = "";
      }
    });

    wsRegions.on('region-clicked', (region, e) => {
      e.stopPropagation(); 
      activeRegion = region; 
      activeTrackId = track.id;
      regionInputStart = region.start.toFixed(2);
      regionInputEnd = region.end.toFixed(2);
      regionError = "";
    });

    ws.on('ready', () => {
      const duration = ws.getDuration();
      if (duration > maxDuration) maxDuration = duration;
    });

    ws.on('interaction', () => {
      const currentTime = ws.getCurrentTime();
      wsInstances.forEach((otherWs, id) => {
        if (id !== track.id && currentTime <= otherWs.getDuration()) otherWs.setTime(currentTime);
      });
    });

    ws.on('scroll', (scrollLeft: number) => {
      globalScrollX = scrollLeft;
      wsInstances.forEach((otherWs, id) => {
        if (id !== track.id) {
          const shadowRoot = otherWs.getWrapper()?.shadowRoot;
          if (shadowRoot) {
             const scrollContainer = shadowRoot.querySelector('.scroll') as HTMLElement;
             if (scrollContainer && scrollContainer.scrollLeft !== scrollLeft) scrollContainer.scrollLeft = scrollLeft;
          }
        }
      });
    });

    ws.on('finish', () => {
      let anyPlaying = false;
      wsInstances.forEach(instance => {
        if (instance.isPlaying()) anyPlaying = true;
      });
      if (!anyPlaying) {
        isPlaying = false;
      }
    });

    return {
      destroy() {
        ws.destroy(); 
        wsInstances.delete(track.id); 
        regionsPlugins.delete(track.id);
      }
    };
  }

  function toggleMute(track: Track) { track.isMuted = !track.isMuted; tracks = [...tracks]; }
  function toggleSolo(track: Track) { track.isSolo = !track.isSolo; tracks = [...tracks]; }
  
  function clearRegion() {
    if (activeTrackId && regionsPlugins.has(activeTrackId)) {
      const plugin = regionsPlugins.get(activeTrackId);
      if (plugin) plugin.clearRegions();
    }
    activeRegion = null; 
    activeTrackId = null;
  }

  async function handleAudioProcess(action: 'cut' | 'trim') {
    if (!activeRegion || activeTrackId === null) return;
    const track = tracks.find(t => t.id === activeTrackId);
    if (!track) return;

    isProcessing = true;
    handleStop(); 

    try {
      const newFilePath = await invoke<string>('process_audio_region', {
        action, filePath: track.realPath, startTime: activeRegion.start, endTime: activeRegion.end
      });

      const baseName = track.displayName.replace(/\.[^/.]+$/, "");
      const newName = `${baseName}_${action === 'cut' ? 'kesildi' : 'kirpildi'}.wav`;
      const assetUrl = convertFileSrc(newFilePath);
      
      const newTheme = action === 'cut' ? cutTheme : trimTheme;
      
      addTrackToUI(newName, assetUrl, newFilePath, newTheme);
      clearRegion();

    } catch (error) {
      alert(`İşlem sırasında bir hata oluştu: ${error}`);
    } finally {
      isProcessing = false;
    }
  }

  onDestroy(() => {
    wsInstances.forEach(ws => ws.destroy());
    wsInstances.clear(); 
    regionsPlugins.clear();
  });
</script>

<svelte:window on:keydown={handleKeydown} on:mousemove={handleMouseMove} on:mouseup={stopResize} />

{#if isProcessing}
  <div class="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex flex-col items-center justify-center">
    <div class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
    <p class="text-text-main mt-4 font-medium tracking-wide">Ses İşleniyor...</p>
    <p class="text-text-muted text-xs mt-2">Lütfen bekleyin...</p>
  </div>
{/if}

<main class="flex-1 flex flex-col font-sans relative overflow-hidden bg-background transition-colors duration-300">
  
  <header class="h-14 bg-background border-b border-border flex items-center justify-between px-6 shadow-sm z-20 shrink-0 transition-colors duration-300">
    <div class="flex items-center gap-4">
      
      <div class="flex items-center gap-2 bg-surface px-3 py-1 rounded-md border border-border transition-colors duration-300">
        <button title="Başa Sar" aria-label="Başa Sar" on:click={handleStop} class="p-1.5 text-text-muted hover:text-text-main hover:bg-border rounded transition-colors disabled:opacity-50">
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><rect x="6" y="6" width="12" height="12"></rect></svg>
        </button>
        <button title="Oynat / Duraklat (Space)" aria-label="Oynat / Duraklat" on:click={handlePlayPause} class="p-1.5 {isPlaying ? 'text-primary' : 'text-text-main'} hover:bg-border rounded transition-colors disabled:opacity-50">
          {#if isPlaying}
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"></path></svg>
          {:else}
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"></path></svg>
          {/if}
        </button>
      </div>

      <div class="flex items-center gap-2 ml-4">
        <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM13 10H7"></path></svg>
        <input title="Yakınlaştır / Uzaklaştır" type="range" min="5" max="150" bind:value={globalZoom} disabled={isProcessing} class="w-24 h-1 bg-border rounded-lg appearance-none cursor-pointer accent-text-muted transition-colors duration-300">
        <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7"></path></svg>
      </div>
    </div>
    
    <div class="flex items-center gap-3">
      <button on:click={triggerFileInput} disabled={isProcessing} class="px-4 py-1.5 text-sm bg-surface hover:bg-border text-text-main border border-border rounded-md transition-all disabled:opacity-50">
        + Ses Yükle
      </button>
      <button on:click={handleExport} disabled={isProcessing || tracks.length === 0} class="px-4 py-1.5 text-sm bg-primary hover:opacity-80 text-white rounded-md font-medium shadow-lg transition-all disabled:opacity-50">
        Mix'i Dışa Aktar
      </button>
    </div>
  </header>

  {#if activeRegion && !isProcessing}
    <div class="absolute bottom-6 left-1/2 -translate-x-1/2 bg-surface border border-border p-2.5 rounded-xl shadow-[0_10px_40px_-10px_rgba(0,0,0,0.8)] z-50 flex flex-col gap-2 animate-in slide-in-from-bottom-4 transition-colors duration-300">
      
      {#if regionError}
        <div class="bg-red-500/10 text-red-400 text-[11px] font-bold px-3 py-1.5 rounded border border-red-500/20 text-center w-full shadow-inner">
          {regionError}
        </div>
      {/if}

      <div class="flex gap-4 items-center w-full">
        <div class="flex flex-col px-2 gap-1.5">
          <span class="text-[10px] text-text-muted uppercase tracking-wider font-bold">Seçili Alan (Saniye)</span>
          <div class="flex items-center gap-2">
            <input type="number" step="0.1" bind:value={regionInputStart} on:input={updateRegionFromInputs} class="w-16 bg-background border border-border hover:border-primary focus:border-primary text-primary font-mono text-sm rounded px-1 py-0.5 outline-none text-center transition-colors">
            <span class="text-text-muted font-bold">-</span>
            <input type="number" step="0.1" bind:value={regionInputEnd} on:input={updateRegionFromInputs} class="w-16 bg-background border border-border hover:border-primary focus:border-primary text-primary font-mono text-sm rounded px-1 py-0.5 outline-none text-center transition-colors">
          </div>
        </div>
        <div class="h-10 w-px bg-border"></div>
        <button title="Seçili alanı çıkar ve kalanları birleştir" on:click={() => handleAudioProcess('cut')} class="px-3 py-2 bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white rounded-lg text-xs transition-colors font-medium flex items-center gap-1.5 h-full">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.121 14.121L19 19m-7-7l7-7m-7 7l-2.879 2.879M12 12L9.121 9.121m0 5.758a3 3 0 10-4.243 4.243 3 3 0 004.243-4.243zm0-5.758a3 3 0 10-4.243-4.243 3 3 0 004.243 4.243z"></path></svg>
          Kes
        </button>
        <button title="Sadece seçili alanı tut, kenarları sil" on:click={() => handleAudioProcess('trim')} class="px-3 py-2 bg-surface border border-border text-text-main hover:bg-border hover:brightness-110 rounded-lg text-xs transition-colors font-medium flex items-center gap-1.5 h-full">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4h16v16H4V4zm2 4h12M6 16h12"></path></svg>
          Kırp
        </button>
        <button title="Seçimi İptal Et" aria-label="İptal" on:click={clearRegion} class="p-2 text-text-muted hover:text-text-main hover:bg-border rounded-lg ml-1 transition-colors h-full">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>
    </div>
  {/if}

  <div class="flex flex-1 overflow-hidden relative">
    
    <div class="flex flex-col shrink-0 bg-surface border-r border-border relative z-20 shadow-[4px_0_24px_-10px_rgba(0,0,0,0.5)] transition-colors duration-300" style="width: {sidebarWidth}px;">
      
      <button type="button" aria-label="Sidebar resize handle" class="absolute top-0 -right-2 bottom-0 w-4 cursor-col-resize z-30 group flex justify-center bg-transparent border-none p-0 outline-none" on:mousedown={startResize} on:keydown={handleResizeKeydown}>
        <div class="w-1 h-full bg-transparent group-hover:bg-primary/30 transition-colors"></div>
      </button>

      <div class="h-8 bg-surface border-b border-border p-2 flex items-center justify-between transition-colors duration-300">
         <span class="text-[10px] text-text-muted uppercase font-bold tracking-wider">Kanallar</span>
      </div>

      <div class="flex-1 overflow-y-auto pb-32">
        {#each tracks as track, index (track.id)}
          <div class="flex h-32 border-b border-border/50 transition-colors {track.isMuted ? 'opacity-50 grayscale' : ''}">
            
            <div class="w-10 flex flex-col border-r border-border/50 bg-background/40 items-center justify-center gap-3 shrink-0 group/order">
              <button on:click={() => moveTrack(index, 'up')} disabled={index === 0} class="text-text-muted hover:text-primary disabled:opacity-20 transition-colors cursor-pointer" title="Kanalı Yukarı Taşı">
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 15l7-7 7 7"></path></svg>
              </button>
              <span class="text-[11px] font-mono font-bold {track.progressColorHex === '#e11d48' ? 'text-rose-500' : track.progressColorHex === '#0891b2' ? 'text-cyan-500' : 'text-emerald-500'} bg-surface px-1.5 py-0.5 rounded shadow-inner border border-border">
                #{index + 1}
              </span>
              <button on:click={() => moveTrack(index, 'down')} disabled={index === tracks.length - 1} class="text-text-muted hover:text-primary disabled:opacity-20 transition-colors cursor-pointer" title="Kanalı Aşağı Taşı">
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 9l-7 7-7-7"></path></svg>
              </button>
            </div>

            <div class="flex-1 p-3 flex flex-col justify-between min-w-0">
              <div class="flex items-center justify-between gap-2">
                 <div class="flex items-center flex-1 gap-2 overflow-hidden">
                   <input 
                     type="text" 
                     bind:value={track.displayName} 
                     class="flex-1 bg-transparent text-sm font-medium text-text-main border border-transparent hover:border-border focus:border-primary focus:bg-background rounded px-1 outline-none transition-all truncate"
                     title="İsmi değiştirmek için tıklayın"
                   />
                 </div>

                 <div class="flex gap-1 shrink-0">
                    <button title="Bu kanalı tek başına dışa aktar" on:click={() => exportSingleTrack(track)} class="w-6 h-6 flex items-center justify-center rounded bg-background text-text-muted hover:bg-primary/20 hover:text-primary transition-colors border border-transparent hover:border-primary/50">
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path></svg>
                    </button>

                    <button title={track.isMuted ? 'Sesi Aç' : 'Sustur (Mute)'} on:click={() => toggleMute(track)} class="w-6 h-6 text-[10px] font-bold rounded transition-colors {track.isMuted ? 'bg-red-500/20 text-red-500 border border-red-500/50' : 'bg-background text-text-muted hover:bg-border hover:text-text-main'}">M</button>
                    <button title={track.isSolo ? 'Solo Modunu Kapat' : 'Sadece Bu Kanalı Dinle (Solo)'} on:click={() => toggleSolo(track)} class="w-6 h-6 text-[10px] font-bold rounded transition-colors {track.isSolo ? 'bg-amber-500/20 text-amber-500 border border-amber-500/50' : 'bg-background text-text-muted hover:bg-border hover:text-text-main'}">S</button>
                    
                    <button title="Kanalı Sil" on:click={() => removeTrack(track.id)} class="w-6 h-6 flex items-center justify-center rounded bg-background text-text-muted hover:bg-red-500/20 hover:text-red-400 transition-colors border border-transparent hover:border-red-500/50">
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
                    </button>
                 </div>
              </div>
              
              <div class="space-y-3">
                <div class="flex items-center gap-2">
                  <span role="button" tabindex="0" title="Çift tıklayarak %80'e sıfırla" class="text-[10px] text-text-muted w-6 cursor-pointer hover:text-primary transition-colors" on:dblclick={() => track.volume = 80} on:keydown={(e) => e.key === 'Enter' && (track.volume = 80)}>VOL</span>
                  <input title="Ses Seviyesi" type="range" min="0" max="100" bind:value={track.volume} on:dblclick={() => track.volume = 80} class="w-full h-1 bg-border rounded-lg appearance-none cursor-pointer accent-primary">
                </div>
                
                <div class="flex items-center justify-between bg-background p-1 rounded-md border border-border transition-colors">
                  <div role="button" tabindex="0" class="flex gap-1" title="Çift tıklayarak merkeze (Center) al" on:dblclick={() => track.pan = 'Center'} on:keydown={(e) => e.key === 'Enter' && (track.pan = 'Center')}>
                    <button class="text-[10px] px-2 py-0.5 rounded transition-colors {track.pan === 'Left' ? 'bg-surface text-text-main border border-border shadow-sm' : 'text-text-muted hover:text-text-main'}" on:click={() => track.pan = 'Left'}>L</button>
                    <button class="text-[10px] px-2 py-0.5 rounded transition-colors {track.pan === 'Center' ? 'bg-surface text-text-main border border-border shadow-sm' : 'text-text-muted hover:text-text-main'}" on:click={() => track.pan = 'Center'}>C</button>
                    <button class="text-[10px] px-2 py-0.5 rounded transition-colors {track.pan === 'Right' ? 'bg-surface text-text-main border border-border shadow-sm' : 'text-text-muted hover:text-text-main'}" on:click={() => track.pan = 'Right'}>R</button>
                  </div>
                </div>
              </div>
            </div>

          </div>
        {/each}
      </div>
    </div>

    <section class="flex-1 flex flex-col relative overflow-hidden">
      <div class="h-8 bg-surface border-b border-border sticky top-0 z-10 text-[10px] text-text-muted select-none overflow-hidden shrink-0 transition-colors duration-300">
        <div class="relative w-full h-full"> 
          <div class="absolute bottom-0 left-4 flex transition-transform duration-75" style="transform: translateX(-{globalScrollX}px);">
            {#each timeMarkers as marker}
              <div class="border-l border-border pl-1 pb-1 shrink-0" style="width: {15 * globalZoom}px">{marker.label}</div>
            {/each}
          </div>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto overflow-x-hidden pb-32">
        {#if tracks.length === 0}
          <div class="flex h-full items-center justify-center p-8">
            <div class="border-2 border-dashed border-border rounded-2xl p-12 text-center max-w-md w-full flex flex-col items-center">
              <svg class="w-16 h-16 text-text-muted/50 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"></path></svg>
              <h3 class="text-lg font-medium text-text-main mb-2">Çalışma Alanı Boş</h3>
              <p class="text-text-muted text-sm mb-6">Miksinize başlamak için sol üst köşedeki butonu kullanarak yeni bir ses dosyası yükleyin.</p>
              <button on:click={triggerFileInput} class="px-5 py-2 text-sm bg-primary hover:opacity-80 text-white rounded-lg font-medium shadow-lg transition-all">
                Ses Dosyası Yükle
              </button>
            </div>
          </div>
        {/if}

        {#each tracks as track (track.id)}
          <div class="flex h-32 border-b border-border/50 bg-surface/20 hover:bg-surface/40 transition-colors {track.isMuted ? 'opacity-50 grayscale' : ''}">
            <div class="flex-1 relative cursor-pointer px-4 py-4 w-full h-full overflow-hidden">
              <div class="absolute inset-0 top-1/2 w-full h-px bg-border/50 -translate-y-1/2 pointer-events-none z-0"></div>
              
              <div class="relative w-full h-20 rounded-md {track.bgClass} border {track.borderClass} z-10 transition-all overflow-hidden group">
                <div use:waveformAction={track} class="w-full h-full"></div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </section>
  </div>
</main>

<style>
  input[type=range]::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    background: var(--color-primary);
    border-radius: 50%;
    cursor: pointer;
  }
  
  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  ::-webkit-scrollbar-track {
    background: var(--color-background); 
  }
  ::-webkit-scrollbar-thumb {
    background: var(--color-border); 
    border-radius: 4px;
    border: 2px solid var(--color-background);
  }
  ::-webkit-scrollbar-thumb:hover {
    background: var(--color-text-muted); 
  }
</style>