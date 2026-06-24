use ratatui::style::Color;

/// 根据标签名生成颜色（使用哈希）
pub fn tag_color(tag: &str) -> Color {
    let mut hash: u32 = 0;
    for byte in tag.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    // 使用预定义的柔和颜色列表
    let colors = [
        Color::Rgb(100, 149, 237), // 蓝色
        Color::Rgb(144, 238, 144), // 绿色
        Color::Rgb(255, 182, 193), // 粉色
        Color::Rgb(255, 218, 185), // 橙色
        Color::Rgb(221, 160, 221), // 紫色
        Color::Rgb(173, 216, 230), // 浅蓝
        Color::Rgb(144, 238, 144), // 浅绿
        Color::Rgb(240, 230, 140), // 黄色
    ];

    colors[(hash as usize) % colors.len()]
}
