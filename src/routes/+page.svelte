<script lang="ts">
  import WaveSurfer from 'wavesurfer.js';
  import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.esm.js';
  import { onDestroy, onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';
  
  import { fade, fly, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';

  interface Theme {
    bgClass: string;
    borderClass: string;
    waveColorVar: string;
    progressColorVar: string;
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
    history: { audioUrl: string; realPath: string; }[];
  }

  type RegionPluginType = ReturnType<typeof RegionsPlugin.create>;
  type SingleRegionType = ReturnType<RegionPluginType['getRegions']>[0];

  const defaultThemes: Theme[] = [
    { bgClass: "bg-success-dim", borderClass: "border-success", waveColorVar: "var(--success)", progressColorVar: "var(--text-main)" },
    { bgClass: "bg-warning-dim", borderClass: "border-warning", waveColorVar: "var(--warning)", progressColorVar: "var(--text-main)" },
    { bgClass: "bg-primary/20", borderClass: "border-primary", waveColorVar: "var(--accent-sec)", progressColorVar: "var(--text-main)" },
  ];

  const cutTheme: Theme = { bgClass: "bg-danger-dim", borderClass: "border-danger", waveColorVar: "var(--danger)", progressColorVar: "var(--text-main)" };
  const trimTheme: Theme = { bgClass: "bg-info-dim", borderClass: "border-info", waveColorVar: "var(--info)", progressColorVar: "var(--text-main)" };

  let tracks: Track[] = [];
  
  const wsInstances = new Map<number, WaveSurfer>();
  const regionsPlugins = new Map<number, RegionPluginType>();

  let audioCtx: AudioContext | null = null;
  const gainNodes = new Map<number, GainNode>();
  const stereoPanners = new Map<number, StereoPannerNode>();
  const mediaSources = new Map<number, MediaElementAudioSourceNode>();

  let isPlaying: boolean = false;
  let isProcessing: boolean = false;
  let globalZoom: number = 20;
  
  let currentPlayingTrackIndex: number | null = null;
  
  let globalScrollX: number = 0;
  let maxDuration: number = 120;
  let timeMarkers: { label: string }[] = [];

  let activeRegion: SingleRegionType | null = null;
  let activeTrackId: number | null = null;

  let regionInputStart: string = "0.00";
  let regionInputEnd: string = "0.00";
  let regionError: string = "";

  let sidebarWidth: number = 320; 
  let isResizing: boolean = false;

  let globalExportFormat: string = "wav";
  let appSettings: any = null;

  function initAudioContext() {
    if (!audioCtx) {
      audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
    }
    if (audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
  }

  onMount(async () => {
    try {
      appSettings = await invoke('ayarlari_getir');
      if (appSettings && appSettings.export_format) {
        globalExportFormat = appSettings.export_format;
      }
    } catch (e) {
      console.error("Ayarlar yüklenemedi:", e);
    }
  });

  $: if (appSettings && globalExportFormat !== appSettings.export_format) {
    appSettings.export_format = globalExportFormat;
    invoke('ayarlari_kaydet', { ayarlar: appSettings }).catch(e => console.error("Ayar kaydedilemedi:", e));
  }

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
    sidebarWidth = Math.min(Math.max(e.clientX, 260), 600);
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
      regionError = "Bitiş süresi başlangıçtan büyük olmalıdır!";
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
    if (wsInstances.size === 0 || isProcessing || tracks.length === 0) return;
    
    initAudioContext();
    isPlaying = !isPlaying;

    if (isPlaying) {
      if (currentPlayingTrackIndex === null) {
        currentPlayingTrackIndex = 0;
      }
      const track = tracks[currentPlayingTrackIndex];
      const ws = wsInstances.get(track.id);
      if (ws) ws.play();
    } else {
      wsInstances.forEach(ws => ws.pause());
    }
  }

  function handleStop() {
    isPlaying = false;
    currentPlayingTrackIndex = null;
    wsInstances.forEach(ws => {
      ws.pause();
      ws.seekTo(0);
    });
  }

  async function triggerFileInput(): Promise<void> {
    try {
      initAudioContext();
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
      history: [], 
      ...theme, 
      audioUrl,
      realPath
    }];
  }

  async function exportSingleTrack(track: Track) {
    try {
      const baseName = track.displayName.replace(/\.[^/.]+$/, "");

      const savePath = await save({
        title: 'Ses Kanalını Dışa Aktar',
        defaultPath: `${baseName}.${globalExportFormat}`,
        filters: [{ name: `${globalExportFormat.toUpperCase()} Ses Dosyası`, extensions: [globalExportFormat] }]
      });

      if (!savePath) return;
      isProcessing = true;

      await invoke('export_single_track', {
        sourcePath: track.realPath,
        destPath: savePath,
        volume: track.volume / 100,
        pan: track.pan
      });

      alert(`${track.displayName} başarıyla ${globalExportFormat.toUpperCase()} olarak dışa aktarıldı!`);
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
        defaultPath: `AudioLoom_Mix.${globalExportFormat}`,
        filters: [{ name: `${globalExportFormat.toUpperCase()} Ses Dosyası`, extensions: [globalExportFormat] }]
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
      alert(`Proje başarıyla ${globalExportFormat.toUpperCase()} formatında dışa aktarıldı!`);

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
      
      const gainNode = gainNodes.get(trackId);
      if (gainNode) gainNode.disconnect();
      gainNodes.delete(trackId);

      const pannerNode = stereoPanners.get(trackId);
      if (pannerNode) pannerNode.disconnect();
      stereoPanners.delete(trackId);
      
      const source = mediaSources.get(trackId);
      if (source) source.disconnect();
      mediaSources.delete(trackId);
    }
    tracks = tracks.filter(t => t.id !== trackId);
    if (tracks.length === 0) {
      isPlaying = false;
      currentPlayingTrackIndex = null;
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

  function undoTrack(trackId: number) {
    const trackIndex = tracks.findIndex(t => t.id === trackId);
    if (trackIndex === -1) return;
    
    const track = tracks[trackIndex];
    if (!track.history || track.history.length === 0) return;

    const previousState = track.history.pop();
    if (previousState) {
      track.realPath = previousState.realPath;
      track.audioUrl = previousState.audioUrl.split('?')[0] + '?t=' + Date.now();
      
      const ws = wsInstances.get(track.id);
      if (ws) {
        ws.load(track.audioUrl);
      }
      
      tracks = [...tracks];
      clearRegion();
    }
  }

  $: {
    const isAnySolo = tracks.some(t => t.isSolo);
    tracks.forEach(track => {
      let finalVolume = track.volume / 100;
      finalVolume = isAnySolo ? ((track.isSolo && !track.isMuted) ? finalVolume : 0) : (track.isMuted ? 0 : finalVolume);
      
      let panCompensation = track.pan === 'Center' ? 1.0 : 1.414; 
      
      const gainNode = gainNodes.get(track.id);
      const pannerNode = stereoPanners.get(track.id);
      const ws = wsInstances.get(track.id);

      if (gainNode && ws) {
        gainNode.gain.value = finalVolume * panCompensation;
        ws.setVolume(1.0); 
      } else if (ws) {
        ws.setVolume(Math.min(finalVolume, 1));
      }

      if (pannerNode) {
        pannerNode.pan.value = track.pan === 'Left' ? -1 : (track.pan === 'Right' ? 1 : 0);
      }
    });
  }

  $: wsInstances.forEach(ws => ws.zoom(globalZoom));

  function waveformAction(node: HTMLElement, track: Track) {
    const ws = WaveSurfer.create({
      container: node, 
      waveColor: track.waveColorVar, 
      progressColor: track.progressColorVar,
      barWidth: 2, barGap: 2, barRadius: 2, height: 96, url: track.audioUrl,
      cursorColor: 'var(--text-main)', cursorWidth: 2, minPxPerSec: globalZoom,
      hideScrollbar: false, autoScroll: true, autoCenter: true, fetchParams: { mode: 'cors' }
    });

    wsInstances.set(track.id, ws);
    const wsRegions = ws.registerPlugin(RegionsPlugin.create());
    regionsPlugins.set(track.id, wsRegions);
    
    wsRegions.enableDragSelection({ color: 'var(--border-hover)' });

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

      try {
        initAudioContext();
        if (audioCtx) {
          if (!mediaSources.has(track.id)) {
            const mediaElement = ws.getMediaElement();
            const source = audioCtx.createMediaElementSource(mediaElement);
            mediaSources.set(track.id, source);

            const gainNode = audioCtx.createGain();
            gainNodes.set(track.id, gainNode);

            const pannerNode = audioCtx.createStereoPanner();
            stereoPanners.set(track.id, pannerNode);

            source.connect(gainNode);
            gainNode.connect(pannerNode);
            pannerNode.connect(audioCtx.destination);
          }

          tracks = [...tracks];
        }
      } catch (e) {
        console.warn("Web Audio API Bağlantı Hatası:", e);
      }
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
      if (isPlaying && currentPlayingTrackIndex !== null) {
        if (tracks[currentPlayingTrackIndex]?.id === track.id) {
          const nextIndex = currentPlayingTrackIndex + 1;
          if (nextIndex < tracks.length) {
            currentPlayingTrackIndex = nextIndex;
            const nextWs = wsInstances.get(tracks[nextIndex].id);
            if (nextWs) nextWs.play();
          } else {
            isPlaying = false;
            currentPlayingTrackIndex = null;
          }
        }
      }
    });

    return {
      destroy() {
        ws.destroy(); 
        wsInstances.delete(track.id); 
        regionsPlugins.delete(track.id);

        const gainNode = gainNodes.get(track.id);
        if (gainNode) gainNode.disconnect();
        gainNodes.delete(track.id);

        const pannerNode = stereoPanners.get(track.id);
        if (pannerNode) pannerNode.disconnect();
        stereoPanners.delete(track.id);

        const source = mediaSources.get(track.id);
        if (source) source.disconnect();
        mediaSources.delete(track.id);
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
    const trackIndex = tracks.findIndex(t => t.id === activeTrackId);
    if (trackIndex === -1) return;
    
    const track = tracks[trackIndex];
    const ws = wsInstances.get(track.id);
    
    isProcessing = true;
    handleStop(); 

    try {
      if (ws) ws.load(''); 

      const newFilePath = await invoke<string>('process_audio_region', {
        action, 
        filePath: track.realPath, 
        startTime: activeRegion.start, 
        endTime: activeRegion.end
      });

      track.history.push({
        audioUrl: track.audioUrl,
        realPath: track.realPath
      });

      track.realPath = newFilePath;
      track.audioUrl = convertFileSrc(newFilePath) + '?t=' + Date.now();

      const newTheme = action === 'cut' ? cutTheme : trimTheme;
      Object.assign(track, newTheme);

      if (ws) {
        ws.setOptions({
          waveColor: track.waveColorVar,
          progressColor: track.progressColorVar
        });
        ws.load(track.audioUrl);
      }

      tracks = [...tracks];
      clearRegion();

    } catch (error) {
      if (ws) ws.load(track.audioUrl);
      alert(`İşlem sırasında bir hata oluştu: ${error}`);
    } finally {
      isProcessing = false;
    }
}

  onDestroy(() => {
    wsInstances.forEach(ws => ws.destroy());
    wsInstances.clear(); 
    regionsPlugins.clear();
    mediaSources.forEach(source => source.disconnect());
    mediaSources.clear();
    gainNodes.forEach(node => node.disconnect());
    stereoPanners.forEach(node => node.disconnect());
  });
</script>

<svelte:window on:keydown={handleKeydown} on:mousemove={handleMouseMove} on:mouseup={stopResize} />

{#if isProcessing}
  <div transition:fade={{ duration: 200 }} class="fixed inset-0 bg-background/90 backdrop-blur-md z-50 flex flex-col items-center justify-center">
    <div class="w-16 h-16 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
    <p class="text-text-main mt-6 text-lg font-semibold tracking-wide">Ses İşleniyor...</p>
    <p class="text-text-muted text-sm mt-2">Lütfen bekleyin, bu işlem bilgisayarınızın hızına göre sürebilir.</p>
  </div>
{/if}

<main class="flex-1 flex flex-col font-sans relative overflow-hidden bg-background transition-colors duration-300 h-screen">
  
  <header class="h-16 bg-surface border-b border-border flex items-center justify-between px-6 shadow-md z-20 shrink-0">
    <div class="flex items-center gap-6">
      
      <div class="flex items-center gap-2 bg-background px-4 py-2 rounded-lg border border-border">
        <button title="Başa Sar" aria-label="Başa Sar" on:click={handleStop} class="p-2 text-text-muted hover:text-text-main hover:bg-surface rounded-md transition-all disabled:opacity-50">
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><rect x="6" y="6" width="12" height="12"></rect></svg>
        </button>
        <div class="w-px h-6 bg-border mx-1"></div>
        <button title="Oynat / Duraklat (Space)" aria-label="Oynat / Duraklat" on:click={handlePlayPause} class="p-2 rounded-md transition-all flex items-center justify-center {isPlaying ? 'bg-primary text-white shadow-lg shadow-primary/30' : 'text-primary hover:bg-primary/10'}">
          {#if isPlaying}
            <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"></path></svg>
          {:else}
            <svg class="w-6 h-6 translate-x-0.5" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"></path></svg>
          {/if}
        </button>
      </div>

      <div class="flex items-center gap-3">
        <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM13 10H7"></path></svg>
        <input title="Yakınlaştır / Uzaklaştır" type="range" min="5" max="150" bind:value={globalZoom} disabled={isProcessing} class="w-32 h-1.5 bg-border rounded-lg appearance-none cursor-pointer accent-primary">
        <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7"></path></svg>
      </div>
    </div>
    
    <div class="flex items-center gap-4">
      <button on:click={triggerFileInput} disabled={isProcessing} class="px-5 py-2.5 text-[15px] font-medium bg-background hover:bg-border text-text-main border border-border rounded-lg transition-all flex items-center gap-2">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        Ses Yükle
      </button>

      <div class="flex items-center bg-background border border-border rounded-lg overflow-hidden transition-all focus-within:border-primary focus-within:ring-2 focus-within:ring-primary/20">
        <select bind:value={globalExportFormat} disabled={isProcessing} aria-label="Format Seçimi" class="bg-transparent text-[15px] text-text-main px-4 py-2.5 outline-none cursor-pointer hover:bg-border font-medium">
          <option value="wav">WAV (Kayıpsız)</option>
          <option value="mp3">MP3 (Sıkıştırılmış)</option>
          <option value="flac">FLAC (Kayıpsız)</option>
          <option value="ogg">OGG (Web)</option>
        </select>
      </div>

      <button on:click={handleExport} disabled={isProcessing || tracks.length === 0} class="px-6 py-2.5 text-[15px] bg-primary hover:bg-primary-sec text-white rounded-lg font-semibold shadow-md shadow-primary/40 hover:shadow-lg hover:shadow-primary/60 hover:-translate-y-0.5 transition-all disabled:opacity-50 disabled:hover:translate-y-0 flex items-center gap-2">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path></svg>
        Dışa Aktar
      </button>
    </div>
  </header>

  {#if activeRegion && !isProcessing}
    <div transition:fly={{ y: 20, duration: 300 }} class="absolute bottom-8 left-1/2 -translate-x-1/2 bg-surface border border-border p-3 rounded-2xl shadow-[0_20px_50px_-12px_rgba(0,0,0,0.8)] z-50 flex flex-col gap-3">
      
      {#if regionError}
        <div transition:slide class="bg-danger-dim text-danger text-xs font-bold px-4 py-2 rounded-lg border border-danger/30 text-center w-full">
          {regionError}
        </div>
      {/if}

      <div class="flex gap-5 items-center w-full px-2">
        <div class="flex flex-col gap-1.5">
          <span class="text-xs text-text-muted uppercase tracking-wider font-semibold">Seçili Alan (Saniye)</span>
          <div class="flex items-center gap-2">
            <input type="number" step="0.1" bind:value={regionInputStart} on:input={updateRegionFromInputs} aria-label="Başlangıç Zamanı" class="w-20 bg-background border border-border focus:border-primary focus:ring-1 focus:ring-primary text-primary font-mono text-base rounded-md px-2 py-1 outline-none text-center transition-all">
            <span class="text-text-muted font-bold">-</span>
            <input type="number" step="0.1" bind:value={regionInputEnd} on:input={updateRegionFromInputs} aria-label="Bitiş Zamanı" class="w-20 bg-background border border-border focus:border-primary focus:ring-1 focus:ring-primary text-primary font-mono text-base rounded-md px-2 py-1 outline-none text-center transition-all">
          </div>
        </div>
        
        <div class="h-12 w-px bg-border"></div>
        
        <button on:click={() => handleAudioProcess('cut')} class="px-4 py-2.5 bg-danger-dim text-danger hover:bg-danger hover:text-white rounded-xl text-sm transition-all font-semibold flex items-center gap-2 shadow-sm">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.121 14.121L19 19m-7-7l7-7m-7 7l-2.879 2.879M12 12L9.121 9.121m0 5.758a3 3 0 10-4.243 4.243 3 3 0 004.243-4.243zm0-5.758a3 3 0 10-4.243-4.243 3 3 0 004.243 4.243z"></path></svg>
          Kes
        </button>
        <button on:click={() => handleAudioProcess('trim')} class="px-4 py-2.5 bg-info-dim border border-info/30 text-info hover:bg-info hover:text-white rounded-xl text-sm transition-all font-semibold flex items-center gap-2 shadow-sm">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4h16v16H4V4zm2 4h12M6 16h12"></path></svg>
          Kırp
        </button>
        
        <button title="İptal Et" aria-label="İptal" on:click={clearRegion} class="p-2.5 text-text-muted hover:text-text-main hover:bg-background rounded-xl ml-2 transition-colors">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>
    </div>
  {/if}

  <div class="flex flex-1 overflow-hidden relative">
    
    <div class="flex flex-col shrink-0 bg-surface border-r border-border relative z-20 shadow-[4px_0_24px_-10px_rgba(0,0,0,0.5)]" style="width: {sidebarWidth}px;">
      
      <button type="button" aria-label="Sidebar resize handle" class="absolute top-0 -right-2 bottom-0 w-4 cursor-col-resize z-30 group flex justify-center bg-transparent border-none p-0 outline-none" on:mousedown={startResize} on:keydown={handleResizeKeydown}>
        <div class="w-1 h-full bg-transparent group-hover:bg-primary/50 transition-colors"></div>
      </button>

      <div class="h-10 bg-surface border-b border-border px-4 flex items-center justify-between shrink-0">
         <span class="text-xs text-text-muted uppercase font-bold tracking-[0.2em]">Ses Kanalları</span>
      </div>

      <div class="flex-1 overflow-y-auto pb-32">
        {#each tracks as track, index (track.id)}
          <div animate:flip={{ duration: 300 }} transition:fade={{ duration: 200 }} class="flex h-36 border-b border-border/50 transition-colors {track.isMuted ? 'opacity-40 grayscale' : ''}">
            
            <div class="w-12 flex flex-col border-r border-border/50 bg-background/30 items-center justify-center gap-4 shrink-0">
              <button 
                aria-label="Kanalı Yukarı Taşı"
                title="Kanalı Yukarı Taşı"
                on:click={() => moveTrack(index, 'up')} 
                disabled={index === 0} 
                class="text-text-muted hover:text-primary disabled:opacity-20 transition-colors cursor-pointer"
              >
                <svg class="w-7 h-7" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 15l7-7 7 7"></path></svg>
              </button>
              <span class="text-xs font-mono font-bold text-text-main bg-background px-2 py-1 rounded-md shadow-inner border border-border">
                {index + 1}
              </span>
              <button 
                aria-label="Kanalı Aşağı Taşı"
                title="Kanalı Aşağı Taşı"
                on:click={() => moveTrack(index, 'down')} 
                disabled={index === tracks.length - 1} 
                class="text-text-muted hover:text-primary disabled:opacity-20 transition-colors cursor-pointer"
              >
                <svg class="w-7 h-7" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 9l-7 7-7-7"></path></svg>
              </button>
            </div>

            <div class="flex-1 p-4 flex flex-col justify-between min-w-0">
              <div class="flex items-center justify-between gap-3 mb-2">
                 <input 
                   type="text" 
                   bind:value={track.displayName} 
                   class="flex-1 bg-transparent text-base font-semibold text-text-main border border-transparent hover:border-border focus:border-primary focus:bg-background rounded-md px-2 py-1 outline-none transition-all truncate"
                   title="Kanal ismini değiştirmek için tıklayın"
                   aria-label="Kanal İsmi"
                 />

                 <div class="flex gap-1.5 shrink-0">
                    {#if track.history && track.history.length > 0}
                      <button title="Geri Al (Undo)" aria-label="Geri Al" on:click={() => undoTrack(track.id)} class="w-8 h-8 flex items-center justify-center rounded-lg bg-warning-dim text-warning hover:bg-warning hover:text-white transition-colors border border-warning/30">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6"></path></svg>
                      </button>
                    {/if}

                    <button title="Tekil Dışa Aktar" aria-label="Tekil Dışa Aktar" on:click={() => exportSingleTrack(track)} class="w-8 h-8 flex items-center justify-center rounded-lg bg-background text-text-muted hover:bg-primary/20 hover:text-primary transition-colors border border-transparent hover:border-primary/30">
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path></svg>
                    </button>

                    <button title={track.isMuted ? 'Sesi Aç' : 'Sustur (Mute)'} aria-label="Sustur" on:click={() => toggleMute(track)} class="w-8 h-8 text-xs font-bold rounded-lg transition-colors {track.isMuted ? 'bg-danger-dim text-danger border border-danger/50' : 'bg-background text-text-muted hover:bg-border hover:text-text-main'}">M</button>
                    <button title={track.isSolo ? 'Solo Modunu Kapat' : 'Sadece Bunu Dinle (Solo)'} aria-label="Solo Dinle" on:click={() => toggleSolo(track)} class="w-8 h-8 text-xs font-bold rounded-lg transition-colors {track.isSolo ? 'bg-warning-dim text-warning border border-warning/50' : 'bg-background text-text-muted hover:bg-border hover:text-text-main'}">S</button>
                    
                    <button title="Sil" aria-label="Sil" on:click={() => removeTrack(track.id)} class="w-8 h-8 flex items-center justify-center rounded-lg bg-background text-text-muted hover:bg-danger-dim hover:text-danger transition-colors border border-transparent hover:border-danger/30 ml-1">
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
                    </button>
                 </div>
              </div>
              
              <div class="space-y-4">
                <div class="flex items-center gap-3 bg-background/50 p-1.5 rounded-lg border border-border/50">
                  <span role="button" tabindex="0" title="Çift tıkla %80 yap" class="text-[11px] font-bold text-text-muted w-8 text-center cursor-pointer hover:text-primary transition-colors" on:dblclick={() => track.volume = 80} on:keydown={(e) => e.key === 'Enter' && (track.volume = 80)}>VOL</span>
                  
                  <input type="range" min="0" max="200" bind:value={track.volume} aria-label="Ses Seviyesi" on:dblclick={() => track.volume = 80} class="w-full h-1.5 bg-border rounded-lg appearance-none cursor-pointer {track.volume > 100 ? 'accent-warning' : 'accent-primary'}">
                  
                  <span class="text-xs font-mono font-bold w-10 text-right {track.volume > 100 ? 'text-warning' : 'text-text-muted'}">
                    %{track.volume}
                  </span>
                </div>
                
                <div class="flex items-center bg-background rounded-lg border border-border p-1 w-max">
                  <button title="Sola Ver" aria-label="Sola Ver" class="text-xs px-3 py-1 rounded-md transition-colors {track.pan === 'Left' ? 'bg-surface text-text-main border border-border shadow-sm' : 'text-text-muted hover:text-text-main'}" on:click={() => track.pan = 'Left'}>L</button>
                  <button title="Merkeze Al" aria-label="Merkeze Al" class="text-xs px-3 py-1 rounded-md transition-colors {track.pan === 'Center' ? 'bg-surface text-text-main border border-border shadow-sm' : 'text-text-muted hover:text-text-main'}" on:click={() => track.pan = 'Center'}>C</button>
                  <button title="Sağa Ver" aria-label="Sağa Ver" class="text-xs px-3 py-1 rounded-md transition-colors {track.pan === 'Right' ? 'bg-surface text-text-main border border-border shadow-sm' : 'text-text-muted hover:text-text-main'}" on:click={() => track.pan = 'Right'}>R</button>
                </div>
              </div>
            </div>

          </div>
        {/each}
      </div>
    </div>

    <section class="flex-1 flex flex-col relative overflow-hidden">
      <div class="h-10 bg-surface border-b border-border sticky top-0 z-10 text-xs text-text-muted select-none overflow-hidden shrink-0">
        <div class="relative w-full h-full"> 
          <div class="absolute bottom-0 left-6 flex transition-transform duration-75" style="transform: translateX(-{globalScrollX}px);">
            {#each timeMarkers as marker}
              <div class="border-l border-border pl-2 pb-1 shrink-0 font-mono" style="width: {15 * globalZoom}px">{marker.label}</div>
            {/each}
          </div>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto overflow-x-hidden pb-32">
        {#if tracks.length === 0}
          <div in:fade class="flex h-full items-center justify-center p-8">
            <div class="border-2 border-dashed border-border rounded-3xl p-16 text-center max-w-lg w-full flex flex-col items-center bg-surface/30">
              <svg class="w-20 h-20 text-text-muted/40 mb-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"></path></svg>
              <h3 class="text-2xl font-bold text-text-main mb-3 tracking-tight">Çalışma Alanı Boş</h3>
              <p class="text-text-muted text-base mb-8 leading-relaxed">Miksinize başlamak için üst menüdeki butonu kullanarak yeni bir ses dosyası yükleyin.</p>
              <button on:click={triggerFileInput} class="px-8 py-3 text-base bg-primary hover:bg-primary-sec text-white rounded-xl font-bold shadow-md shadow-primary/40 hover:-translate-y-1 transition-all">
                Ses Dosyası Yükle
              </button>
            </div>
          </div>
        {/if}

        {#each tracks as track (track.id)}
          <div class="flex h-36 border-b border-border/50 bg-surface/20 hover:bg-surface/60 transition-colors {track.isMuted ? 'opacity-40 grayscale' : ''}">
            <div class="flex-1 relative cursor-pointer px-6 py-4 w-full h-full overflow-hidden">
              <div class="absolute inset-0 top-1/2 w-full h-px bg-border/40 -translate-y-1/2 pointer-events-none z-0"></div>
              
              <div class="relative w-full h-28 rounded-xl {track.bgClass} border-2 {track.borderClass} {currentPlayingTrackIndex !== null && tracks[currentPlayingTrackIndex]?.id === track.id ? 'border-primary shadow-[0_0_15px_rgba(var(--accent),0.5)]' : 'border-transparent'} z-10 transition-all overflow-hidden group">
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
  ::-webkit-scrollbar { width: 10px; height: 10px; }
  ::-webkit-scrollbar-track { background: var(--color-background); }
  ::-webkit-scrollbar-thumb {
    background: var(--color-border); 
    border-radius: 6px;
    border: 3px solid var(--color-background);
  }
  ::-webkit-scrollbar-thumb:hover { background: var(--color-text-muted); }

  input[type=range]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    cursor: pointer;
    box-shadow: 0 0 10px rgba(0,0,0,0.5);
    transition: transform 0.1s;
  }
  input[type=range]::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }
</style>