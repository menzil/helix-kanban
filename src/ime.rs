/// 输入法切换模块
///
/// 在正常模式下使用英文输入法，在对话框模式下使用用户的默认输入法

use std::process::Command;

/// 输入法状态
pub struct ImeState {
    /// 进入对话框前的输入法
    previous_ime: Option<String>,
    /// 英文输入法标识
    english_ime: String,
}

impl ImeState {
    pub fn new() -> Self {
        // macOS 常见的英文输入法
        let english_ime = if cfg!(target_os = "macos") {
            "com.apple.keylayout.ABC".to_string()
        } else {
            // Linux 等其他系统可以扩展
            String::new()
        };

        Self {
            previous_ime: None,
            english_ime,
        }
    }

    /// 切换到英文输入法（在进入正常模式时调用）
    pub fn switch_to_english(&mut self) {
        if !self.english_ime.is_empty() {
            let _ = self.switch_ime(&self.english_ime);
        }
    }

    /// 进入对话框时：保存当前输入法（应该是英文）并恢复用户之前的输入法
    pub fn enter_dialog(&mut self) {
        // 如果有保存的用户输入法，恢复它
        if let Some(ref ime) = self.previous_ime.clone() {
            let _ = self.switch_ime(ime);
        }
        // 如果没有保存的，就保持当前输入法（用户可以自己切换）
    }

    /// 退出对话框时：保存当前输入法（可能是用户的中文输入法）并切换到英文
    pub fn exit_dialog(&mut self) {
        // 保存当前输入法（用户在对话框中可能切换到了中文）
        if let Some(current) = self.get_current_ime() {
            // 只有不是英文输入法时才保存
            if current != self.english_ime {
                self.previous_ime = Some(current);
            }
        }

        // 切换到英文
        self.switch_to_english();
    }

    /// 保存当前输入法并切换到英文（在进入对话框前调用）
    pub fn save_and_switch_to_english(&mut self) {
        // 保存当前输入法
        if let Some(current) = self.get_current_ime() {
            self.previous_ime = Some(current);
        }

        // 切换到英文
        self.switch_to_english();
    }

    /// 恢复之前保存的输入法（在退出对话框时调用）
    pub fn restore_previous(&mut self) {
        if let Some(ref ime) = self.previous_ime.clone() {
            let _ = self.switch_ime(ime);
            self.previous_ime = None;
        }
    }

    /// 获取当前输入法
    fn get_current_ime(&self) -> Option<String> {
        if cfg!(target_os = "macos") {
            // 尝试使用 im-select 工具
            if let Ok(output) = Command::new("im-select").output() {
                if output.status.success() {
                    return String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string());
                }
            }
        }
        None
    }

    /// 切换输入法
    fn switch_ime(&self, ime_id: &str) -> Result<(), String> {
        if cfg!(target_os = "macos") {
            // 尝试使用 im-select 工具
            match Command::new("im-select").arg(ime_id).output() {
                Ok(output) => {
                    if output.status.success() {
                        Ok(())
                    } else {
                        Err("切换输入法失败".to_string())
                    }
                }
                Err(_) => {
                    // im-select 未安装，静默忽略
                    Ok(())
                }
            }
        } else {
            // 其他系统暂不支持
            Ok(())
        }
    }
}

impl Default for ImeState {
    fn default() -> Self {
        Self::new()
    }
}
