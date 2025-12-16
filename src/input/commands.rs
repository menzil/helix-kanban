/// 应用命令枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // ===== 退出 =====
    Quit,

    // ===== 分屏管理 (Space w 前缀) =====
    /// 水平分割当前面板
    SplitHorizontal,
    /// 垂直分割当前面板
    SplitVertical,
    /// 关闭当前面板
    ClosePane,
    /// 切换到下一个窗口
    FocusNextPane,
    /// 聚焦左侧面板
    FocusLeft,
    /// 聚焦右侧面板
    FocusRight,
    /// 聚焦上方面板
    FocusUp,
    /// 聚焦下方面板
    FocusDown,
    /// 最大化/恢复当前面板
    MaximizePane,

    // ===== 任务操作 =====
    /// 将任务移到左边的状态列
    MoveTaskLeft,
    /// 将任务移到右边的状态列
    MoveTaskRight,
    /// 任务在当前列中上移
    MoveTaskUp,
    /// 任务在当前列中下移
    MoveTaskDown,
    /// 选择上一个任务
    TaskUp,
    /// 选择下一个任务
    TaskDown,
    /// 切换到左边的列
    ColumnLeft,
    /// 切换到右边的列
    ColumnRight,
    /// 删除当前任务
    DeleteTask,
    /// 创建新任务
    NewTask,
    /// 用外部编辑器创建新任务
    NewTaskInEditor,
    /// 编辑当前任务
    EditTask,
    /// 用外部编辑器编辑任务
    EditTaskInEditor,
    /// 预览任务（内部 TUI）
    ViewTask,
    /// 用外部工具预览任务
    ViewTaskExternal,
    /// 复制任务到剪贴板
    CopyTask,
    /// 设置任务优先级
    SetTaskPriority(String),  // "high", "medium", "low", "none"

    // ===== 项目操作 (Space p 前缀) =====
    /// 打开项目
    OpenProject,
    /// 创建新项目（根据上下文决定全局或本地）
    NewProject,
    /// 创建新的本地项目
    NewLocalProject,
    /// 创建新的全局项目
    NewGlobalProject,
    /// 隐藏项目（软删除 - 只对全局项目）
    HideProject,
    /// 删除项目（硬删除 - 删除文件）
    DeleteProject,
    /// 重命名项目
    RenameProject,
    /// 重新加载当前项目
    ReloadCurrentProject,
    /// 重新加载所有项目
    ReloadAllProjects,

    // ===== 模式切换 =====
    /// 进入命令模式
    EnterCommandMode,
    /// 进入正常模式
    EnterNormalMode,
    /// 取消当前操作
    Cancel,
}
