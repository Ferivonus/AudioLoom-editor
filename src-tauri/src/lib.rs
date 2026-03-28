use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use tauri::Manager;
use serde::{ Serialize};
use std::fs;
use tauri::{AppHandle};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Serialize, Deserialize, Clone)]
pub struct Ayarlar {
    pub export_format: String,
    pub base_export_dir: String,
    pub folder_format: String,
    pub custom_prefix: String,
    pub sub_folder: String,
    pub ask_every_time: bool,
    pub current_theme: String,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Self {
            export_format: "wav".to_string(),
            base_export_dir: "".to_string(),
            folder_format: "DD-MM-YYYY".to_string(),
            custom_prefix: "audioloom-editor".to_string(),
            sub_folder: "Mixler".to_string(),
            ask_every_time: true,
            current_theme: "theme-modern".to_string(),
        }
    }
}

#[tauri::command]
fn get_app_data_dir(app: AppHandle) -> Result<String, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
fn ayarlari_getir(app: AppHandle) -> Result<Ayarlar, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let settings_path = dir.join("audioloom_settings.json");

    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    if settings_path.exists() {
        let data = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        
        match serde_json::from_str::<Ayarlar>(&data) {
            Ok(settings) => Ok(settings),
            Err(_) => {
                let varsayilan = Ayarlar::default();
                if let Ok(yeni_data) = serde_json::to_string_pretty(&varsayilan) {
                    let _ = fs::write(&settings_path, yeni_data);
                }
                Ok(varsayilan)
            }
        }
    } else {
        let varsayilan = Ayarlar::default();
        let data = serde_json::to_string_pretty(&varsayilan).map_err(|e| e.to_string())?;
        fs::write(&settings_path, data).map_err(|e| e.to_string())?;
        
        Ok(varsayilan)
    }
}

#[tauri::command]
fn ayarlari_kaydet(app: AppHandle, ayarlar: Ayarlar) -> Result<(), String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    
    let settings_path = dir.join("audioloom_settings.json");
    let data = serde_json::to_string_pretty(&ayarlar).map_err(|e| e.to_string())?;
    
    fs::write(settings_path, data).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_ffmpeg_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map(|p| p.parent().map(|p| p.to_path_buf()).unwrap_or_default())
        .unwrap_or_default();
    
    let resource_dir = app.path().resource_dir().unwrap_or_default();
    let current_dir = std::env::current_dir().unwrap_or_default();

    let olasi_yollar = vec![
        exe_path.join("binaries").join("ffmpeg.exe"),
        resource_dir.join("binaries").join("ffmpeg.exe"),
        current_dir.join("src-tauri").join("binaries").join("ffmpeg.exe"),
        current_dir.join("binaries").join("ffmpeg.exe"),
    ];

    olasi_yollar.into_iter().find(|p| p.exists())
        .ok_or_else(|| "FFmpeg aracı 'binaries' klasöründe bulunamadı!".to_string())
}

#[tauri::command]
async fn process_audio_region(
    app: tauri::AppHandle,
    action: String,
    file_path: String,
    start_time: f64,
    end_time: f64,
) -> Result<String, String> {
    
    tauri::async_runtime::spawn_blocking(move || {
        let ffmpeg_path = get_ffmpeg_path(&app)?;
        let source_path = Path::new(&file_path);
        
        if !source_path.exists() {
            return Err(format!("Kaynak dosya bulunamadı: {}", file_path));
        }

        let file_stem = source_path.file_stem().unwrap_or_default().to_string_lossy();
        let extension = source_path.extension().unwrap_or_default().to_string_lossy();
        
        let local_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
        let cache_dir = local_data_dir.join("AudioLoom_Cache");
        
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).map_err(|e| format!("Önbellek klasörü oluşturulamadı: {}", e))?;
        }
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let output_file_name = format!("{}_{}_{}.{}", file_stem, action, timestamp, extension);
        let output_path = cache_dir.join(output_file_name);

        let mut cmd = Command::new(&ffmpeg_path);

        if action == "trim" {
            cmd.arg("-y")
               .arg("-i").arg(&source_path)
               .arg("-ss").arg(start_time.to_string())
               .arg("-to").arg(end_time.to_string())
               .arg("-c").arg("copy")
               .arg(&output_path);
        } else if action == "cut" {
            let filter = format!(
                "[0:a]atrim=end={start},asetpts=PTS-STARTPTS[a1];[0:a]atrim=start={end},asetpts=PTS-STARTPTS[a2];[a1][a2]concat=n=2:v=0:a=1[out]",
                start = start_time,
                end = end_time
            );

            cmd.arg("-y")
               .arg("-i").arg(&source_path)
               .arg("-filter_complex").arg(filter)
               .arg("-map").arg("[out]")
               .arg(&output_path);
        } else {
            return Err("Bilinmeyen işlem türü.".to_string());
        }

        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let output = cmd.output().map_err(|e| format!("FFmpeg çalıştırılamadı: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("FFmpeg İşlem Hatası: {}", stderr));
        }

        Ok(output_path.to_string_lossy().to_string())

    }).await.map_err(|e| format!("İşlem çöktü: {}", e))?
}

#[derive(Deserialize)]
struct TrackInfo {
    path: String,
    volume: f64,
    pan: String,
}

#[tauri::command]
async fn export_project(
    app: tauri::AppHandle,
    tracks: Vec<TrackInfo>,
    output_path: String,
) -> Result<String, String> {
    
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(parent) = Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Klasör oluşturulamadı: {}", e))?;
        }

        let ffmpeg_path = get_ffmpeg_path(&app)?;
        let mut cmd = Command::new(&ffmpeg_path);

        for track in &tracks {
            cmd.arg("-i").arg(&track.path);
        }

        let mut filter_complex = String::new();
        let mut concat_inputs = String::new();

        for (i, track) in tracks.iter().enumerate() {
            let pan_filter = match track.pan.as_str() {
                "Left" => "stereo|c0=FL|c1=0",
                "Right" => "stereo|c0=0|c1=FR",
                _ => "stereo|c0=FL|c1=FR",
            };

            filter_complex.push_str(&format!(
                "[{}:a]volume={},pan={}[a{}];", 
                i, track.volume, pan_filter, i
            ));
            
            concat_inputs.push_str(&format!("[a{}]", i));
        }

        filter_complex.push_str(&format!(
            "{}concat=n={}:v=0:a=1[out]", 
            concat_inputs, tracks.len()
        ));

        cmd.arg("-filter_complex").arg(&filter_complex)
           .arg("-map").arg("[out]")
           .arg("-y")
           .arg(&output_path);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);

        let output = cmd.output().map_err(|e| format!("FFmpeg çalıştırılamadı: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("FFmpeg Dışa Aktarma Hatası: {}", stderr));
        }

        Ok("Başarılı".to_string())
    }).await.map_err(|e| format!("İşlem zaman aşımına uğradı: {}", e))?
}

#[tauri::command]
async fn export_single_track(
    app: tauri::AppHandle,
    source_path: String,
    dest_path: String,
    volume: f64,
    pan: String,
) -> Result<(), String> {
    
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(parent) = Path::new(&dest_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Klasör oluşturulamadı: {}", e))?;
        }

        let ffmpeg_path = get_ffmpeg_path(&app)?;
        let mut cmd = Command::new(&ffmpeg_path);

        let pan_filter = match pan.as_str() {
            "Left" => "stereo|c0=FL|c1=0",
            "Right" => "stereo|c0=0|c1=FR",
            _ => "stereo|c0=FL|c1=FR",
        };

        let filter = format!("volume={},pan={}", volume, pan_filter);

        cmd.arg("-y")
           .arg("-i").arg(&source_path)
           .arg("-af").arg(filter)
           .arg(&dest_path);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); 

        let output = cmd.output().map_err(|e| format!("FFmpeg çalıştırılamadı: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Dosya aktarım hatası: {}", stderr));
        }

        Ok(())
    }).await.map_err(|e| format!("İşlem çöktü: {}", e))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init()) 
        .plugin(tauri_plugin_fs::init())     
        .invoke_handler(tauri::generate_handler![process_audio_region, export_project, export_single_track, get_app_data_dir, ayarlari_getir, ayarlari_kaydet])
        .run(tauri::generate_context!())
        .expect("Tauri uygulaması çalıştırılırken bir hata oluştu");
}