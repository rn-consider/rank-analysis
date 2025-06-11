// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use lol_record_analysis_tauri_lib::ipc;
use std::process::Command;
use std::sync::Mutex;
use tauri::Manager;

// 添加这一行来导入 CommandExt trait
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

struct BackendProcess(Mutex<Option<std::process::Child>>);

fn start_backend_process() -> std::process::Child {
    let exe_path = std::env::current_exe()
        .expect("Failed to get current exe path")
        .parent()
        .expect("Failed to get parent directory")
        .join("lol-record-analysis.exe");

    #[cfg(target_os = "windows")]
    {
        Command::new(exe_path)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW 标志
            .spawn()
            .expect("Failed to start backend process")
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Command::new(exe_path)
            .spawn()
            .expect("Failed to start backend process")
    }
}

// 强制清理所有相关进程
#[cfg(target_os = "windows")]
fn force_cleanup_processes() {
    use std::thread;
    use std::time::Duration;
    
    // 给进程一点时间正常退出
    thread::sleep(Duration::from_millis(500));
    
    // 杀死所有可能残留的 Edge WebView2 进程
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "msedgewebview2.exe"])
        .output();
    
    // 杀死所有可能残留的后端进程
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "lol-record-analysis.exe"])
        .output();
        
    // 杀死所有可能残留的 Edge 相关进程
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "msedge.exe"])
        .output();
        
    // 使用 wmic 终止所有 WebView2 相关的进程树
    let _ = Command::new("wmic")
        .args(&["process", "where", "name='msedgewebview2.exe'", "delete"])
        .output();
        
    // 额外的安全措施：终止包含特定参数的进程
    let current_exe = std::env::current_exe().unwrap_or_default();
    let exe_name = current_exe.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("lol-record-analysis-tauri");
        
    // 杀死与当前应用程序相关的进程
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", &format!("{}.exe", exe_name)])
        .output();
}

#[cfg(not(target_os = "windows"))]
fn force_cleanup_processes() {
    // 非 Windows 平台的清理逻辑（如果需要）
}

fn main() {
    // 注册退出时的清理处理器
    #[cfg(target_os = "windows")]
    {
        ctrlc::set_handler(move || {
            force_cleanup_processes();
            std::process::exit(0);
        }).expect("Error setting Ctrl-C handler");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        // 添加库的命令插件
        .invoke_handler(tauri::generate_handler![ipc::get_summoner, ipc::cleanup_processes])
        // 添加进程管理
        .manage(BackendProcess(Mutex::new(None)))
        .setup(|app| {
            if !cfg!(debug_assertions) {
                let process = start_backend_process();
                *app.state::<BackendProcess>().0.lock().unwrap() = Some(process);
            }
            Ok(())
        })
        .on_window_event(|app_handle, event| {
            match event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    // 清理后端进程
                    if let Some(mut process) = app_handle
                        .state::<BackendProcess>()
                        .0
                        .lock()
                        .unwrap()
                        .take()
                    {
                        let _ = process.kill();
                        let _ = process.wait(); // 等待进程完全退出
                    }
                    
                    // Windows 下强制清理所有相关进程
                    #[cfg(target_os = "windows")]
                    force_cleanup_processes();
                },
                tauri::WindowEvent::Destroyed => {
                    // 窗口销毁时再次确保清理
                    #[cfg(target_os = "windows")]
                    force_cleanup_processes();
                },
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
