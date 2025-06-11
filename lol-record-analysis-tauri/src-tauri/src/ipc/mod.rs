use crate::lcu::api::summoner::Summoner;

#[tauri::command]
pub async fn get_summoner(source_type: &str, source_id: &str) -> Result<Summoner, String> {
    match source_type {
        "my" => {
            // 1. 调用 async 函数时加上 .await
            // 2. 确保返回 Ok(summoner)
            let summoner = Summoner::get_my_summoner().await?;
            Ok(summoner)
        }
        "puuid" => {
            // 调用 async 函数时加上 .await
            let summoner = Summoner::new_by_puuid(source_id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(summoner)
        }
        "name" => {
            // 调用 async 函数时加上 .await
            let summoner = Summoner::new_by_name(source_id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(summoner)
        }
        _ => {
            let summoner = Summoner::get_my_summoner().await?;
            Ok(summoner)
        }
    }
}

#[tauri::command]
pub async fn cleanup_processes() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        // 强制清理所有相关进程
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "msedgewebview2.exe"])
            .output();

        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "lol-record-analysis.exe"])
            .output();

        Ok("进程清理完成".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("非 Windows 系统，无需清理".to_string())
    }
}
