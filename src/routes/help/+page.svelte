<script lang="ts">

</script>

<svelte:head>
  <title>Yardım - AudioLoom</title>
</svelte:head>

<div class="flex-1 overflow-y-auto p-10 bg-background font-sans transition-colors duration-500">
  <div class="max-w-5xl mx-auto pb-32">
    <div class="mb-16">
      <h2 class="text-5xl lg:text-6xl font-black uppercase italic text-text-main tracking-tighter leading-none">
        Kullanım Kılavuzu <br/> <span class="text-primary">& Yardım</span>
      </h2>
      <p class="text-text-muted text-lg mt-6 font-medium leading-relaxed max-w-2xl">
        AudioLoom'un profesyonel araçlarını keşfedin ve ses projelerinizi yüksek standartlarda yönetin.
      </p>
    </div>
    
    <div class="grid gap-12">
      
      <section class="bg-surface border border-border rounded-[2.5rem] p-10 shadow-2xl transition-colors">
        <h3 class="text-2xl font-black uppercase italic text-primary mb-8 flex items-center gap-4 border-b border-border/50 pb-6">
          <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z"></path>
          </svg>
          Temel Kısayollar ve İpuçları
        </h3>
        
        <ul class="space-y-6">
          <li class="flex justify-between items-center border-b border-border/30 pb-6">
            <span class="text-text-main text-lg font-bold">Oynat / Duraklat</span>
            <kbd class="bg-background px-4 py-2 rounded-xl border border-border font-mono text-sm font-black text-primary shadow-inner uppercase tracking-widest">Space</kbd>
          </li>
          <li class="flex justify-between items-center border-b border-border/30 pb-6">
            <div class="flex flex-col gap-1">
              <span class="text-text-main text-lg font-bold">Sesi Varsayılana Sıfırla (%80)</span>
              <span class="text-sm text-text-muted font-medium opacity-70">Ses çubuğunun yanındaki VOL etiketine uygulanır</span>
            </div>
            <span class="text-primary font-black uppercase text-xs tracking-widest bg-primary/10 px-4 py-2 rounded-xl border border-primary/20">Çift Tıkla</span>
          </li>
          <li class="flex justify-between items-center border-b border-border/30 pb-6">
            <div class="flex flex-col gap-1">
              <span class="text-text-main text-lg font-bold">Sesi Merkeze (Center) Al</span>
              <span class="text-sm text-text-muted font-medium opacity-70">L-C-R yazan buton grubuna uygulanır</span>
            </div>
            <span class="text-primary font-black uppercase text-xs tracking-widest bg-primary/10 px-4 py-2 rounded-xl border border-primary/20">Çift Tıkla</span>
          </li>
          <li class="flex justify-between items-center">
            <div class="flex flex-col gap-1">
              <span class="text-text-main text-lg font-bold">Panel Genişliğini Ayarla</span>
              <span class="text-sm text-text-muted font-medium opacity-70">Kanallar menüsünün sağ kenarından sürükleyerek genişliği değiştirebilirsiniz</span>
            </div>
            <span class="text-primary font-black uppercase text-xs tracking-widest bg-primary/10 px-4 py-2 rounded-xl border border-primary/20">Kenarı Sürükle</span>
          </li>
        </ul>
      </section>

      <section class="bg-surface border border-border rounded-[2.5rem] p-10 shadow-2xl transition-colors">
        <h3 class="text-2xl font-black uppercase italic text-primary mb-8 flex items-center gap-4 border-b border-border/50 pb-6">
          <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h7"></path>
          </svg>
          Kanal Yönetim Protokolü
        </h3>
        
        <div class="grid grid-cols-1 md:grid-cols-2 gap-10">
          <div class="space-y-3">
            <h4 class="text-text-main text-xl font-black uppercase tracking-tight flex items-center gap-3">
              <span class="w-3 h-3 rounded-full bg-primary shadow-[0_0_12px_var(--accent)]"></span> İsim Değiştirme
            </h4>
            <p class="text-text-muted text-base leading-relaxed font-medium">Kanalların ismini özelleştirmek için isim alanına tıklayıp yeni bir kimlik tanımlayabilirsiniz.</p>
          </div>
          <div class="space-y-3">
            <h4 class="text-text-main text-xl font-black uppercase tracking-tight flex items-center gap-3">
              <span class="w-3 h-3 rounded-full bg-primary shadow-[0_0_12px_var(--accent)]"></span> Sıralama Sistemi
            </h4>
            <p class="text-text-muted text-base leading-relaxed font-medium">Ok ikonlarını kullanarak kanalları dikey düzlemde yukarı veya aşağı taşıyarak miks hiyerarşisini düzenleyebilirsiniz.</p>
          </div>
          <div class="space-y-3">
            <h4 class="text-text-main text-xl font-black uppercase tracking-tight flex items-center gap-3">
              <span class="w-3 h-3 rounded-full bg-primary shadow-[0_0_12px_var(--accent)]"></span> Solo & Mute
            </h4>
            <p class="text-text-muted text-base leading-relaxed font-medium"><strong class="text-amber-500 font-black">S (Solo)</strong> ile sadece seçili kanala odaklanabilir, <strong class="text-rose-500 font-black">M (Mute)</strong> ile kanalı geçici olarak devre dışı bırakabilirsiniz.</p>
          </div>
          <div class="space-y-3">
            <h4 class="text-text-main text-xl font-black uppercase tracking-tight flex items-center gap-3">
              <span class="w-3 h-3 rounded-full bg-primary shadow-[0_0_12px_var(--accent)]"></span> Panoramatik Ayar
            </h4>
            <p class="text-text-muted text-base leading-relaxed font-medium">Sesi stereo düzlemde (Sol, Merkez, Sağ) konumlandırarak mekânsal bir derinlik oluşturabilirsiniz.</p>
          </div>
        </div>
      </section>

      <section class="bg-surface border border-border rounded-[2.5rem] p-10 shadow-2xl transition-colors">
        <h3 class="text-2xl font-black uppercase italic text-primary mb-8 flex items-center gap-4 border-b border-border/50 pb-6">
          <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" d="M14.121 14.121L19 19m-7-7l7-7m-7 7l-2.879 2.879M12 12L9.121 9.121m0 5.758a3 3 0 10-4.243 4.243 3 3 0 004.243-4.243zm0-5.758a3 3 0 10-4.243-4.243 3 3 0 004.243 4.243z"></path>
          </svg>
          Hassas Düzenleme Modülleri
        </h3>
        
        <div class="space-y-10">
          <div>
            <h4 class="text-text-main text-xl font-black uppercase tracking-tight mb-3">Seçim ve Kalibrasyon</h4>
            <p class="text-text-muted text-lg leading-relaxed font-medium">
              Dalga formu üzerinde sürükleme yaparak bir bölge tanımlayın. Alt paneldeki <code class="bg-background px-3 py-1 rounded-xl border border-border text-primary font-mono font-black text-sm">Saniye</code> kutucuklarını kullanarak milisaniyelik hassasiyetle seçim aralığını güncelleyebilirsiniz.
            </p>
          </div>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="p-8 bg-background rounded-4xl border border-border/50 shadow-inner relative overflow-hidden group">
              <h4 class="font-black uppercase tracking-[0.2em] text-rose-500 flex items-center gap-4 mb-4 text-lg">
                <span class="w-5 h-5 rounded-lg bg-rose-500 shadow-[0_0_15px_rgba(244,63,94,0.8)]"></span> Kesme (Cut)
              </h4>
              <p class="text-text-muted text-base leading-relaxed font-medium italic">Seçilen aralığı projeden ayıklar ve kalan parçaları otomatik olarak birleştirir. <span class="text-rose-400 font-bold">Kırmızı renk</span> ile sembolize edilir.</p>
            </div>
            
            <div class="p-8 bg-background rounded-4xl border border-border/50 shadow-inner relative overflow-hidden group">
              <h4 class="font-black uppercase tracking-[0.2em] text-cyan-500 flex items-center gap-4 mb-4 text-lg">
                <span class="w-5 h-5 rounded-lg bg-cyan-500 shadow-[0_0_15px_rgba(6,182,212,0.8)]"></span> Kırpma (Trim)
              </h4>
              <p class="text-text-muted text-base leading-relaxed font-medium italic">Sadece seçilen bölgeyi muhafaza eder, dışta kalan kısımları temizler. <span class="text-cyan-400 font-bold">Mavi renk</span> ile sembolize edilir.</p>
            </div>
          </div>
        </div>
      </section>

      <section class="bg-surface border border-border rounded-[2.5rem] p-10 shadow-2xl transition-colors">
        <h3 class="text-2xl font-black uppercase italic text-primary mb-8 flex items-center gap-4 border-b border-border/50 pb-6">
          <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path>
          </svg>
          Mastering & Dışa Aktarım
        </h3>
        <div class="space-y-8">
          <div class="border-l-4 border-primary pl-8 py-2">
            <h4 class="text-text-main text-xl font-black uppercase tracking-tight">Tekil Kanal Aktarımı</h4>
            <p class="text-text-muted text-lg mt-2 leading-relaxed font-medium">Kanal isminin yanındaki <strong class="text-primary uppercase tracking-widest text-sm">Aktar</strong> butonuyla sadece o kanala özel efekt ve ayarları içeren dosyayı alabilirsiniz.</p>
          </div>
          <div class="border-l-4 border-primary pl-8 py-2">
            <h4 class="text-text-main text-xl font-black uppercase tracking-tight">Master Mix Aktarımı</h4>
            <p class="text-text-muted text-lg mt-2 leading-relaxed font-medium">Tüm projeyi, Solo ve Mute ayarlarını koruyarak tek bir yüksek kaliteli <code>.wav</code> dosyasına dönüştürmek için <span class="text-background bg-primary px-4 py-1.5 rounded-xl text-xs font-black uppercase tracking-[0.2em] shadow-lg ml-2">Mix'i Aktar</span> komutunu kullanın.</p>
          </div>
        </div>
      </section>

    </div>
  </div>
</div>

<style>
  div::-webkit-scrollbar {
    width: 8px;
  }
  div::-webkit-scrollbar-track {
    background: transparent;
  }
  div::-webkit-scrollbar-thumb {
    background: var(--color-border);
    border-radius: 20px;
    border: 2px solid var(--color-background);
  }
  div::-webkit-scrollbar-thumb:hover {
    background: var(--color-primary);
  }
</style>