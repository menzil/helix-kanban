use serde::{Deserialize, Serialize};

/// 分屏节点 - 支持递归分屏
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitNode {
    /// 叶子节点 - 显示单个项目
    Leaf {
        /// 当前显示的项目ID
        project_id: Option<String>,
        /// 面板ID
        id: usize,
    },
    /// 水平分割 - 左右两个子面板
    Horizontal {
        left: Box<SplitNode>,
        right: Box<SplitNode>,
        /// 左侧面板占比 (0.0 - 1.0)
        ratio: f32,
    },
    /// 垂直分割 - 上下两个子面板
    Vertical {
        top: Box<SplitNode>,
        bottom: Box<SplitNode>,
        /// 上方面板占比 (0.0 - 1.0)
        ratio: f32,
    },
}

impl SplitNode {
    /// 创建新的叶子节点
    pub fn new_leaf(id: usize) -> Self {
        SplitNode::Leaf {
            project_id: None,
            id,
        }
    }

    /// 水平分割当前节点
    pub fn split_horizontal(&mut self, new_id: usize) {
        let old = std::mem::replace(self, SplitNode::new_leaf(0));
        *self = SplitNode::Horizontal {
            left: Box::new(old),
            right: Box::new(SplitNode::new_leaf(new_id)),
            ratio: 0.5,
        };
    }

    /// 垂直分割当前节点
    pub fn split_vertical(&mut self, new_id: usize) {
        let old = std::mem::replace(self, SplitNode::new_leaf(0));
        *self = SplitNode::Vertical {
            top: Box::new(old),
            bottom: Box::new(SplitNode::new_leaf(new_id)),
            ratio: 0.5,
        };
    }

    /// 查找指定ID的面板（可变引用）
    pub fn find_pane_mut(&mut self, id: usize) -> Option<&mut SplitNode> {
        match self {
            SplitNode::Leaf { id: leaf_id, .. } if *leaf_id == id => Some(self),
            SplitNode::Horizontal { left, right, .. } => {
                left.find_pane_mut(id).or_else(|| right.find_pane_mut(id))
            }
            SplitNode::Vertical { top, bottom, .. } => {
                top.find_pane_mut(id).or_else(|| bottom.find_pane_mut(id))
            }
            _ => None,
        }
    }

    /// 查找指定ID的面板（不可变引用）
    pub fn find_pane(&self, id: usize) -> Option<&SplitNode> {
        match self {
            SplitNode::Leaf { id: leaf_id, .. } if *leaf_id == id => Some(self),
            SplitNode::Horizontal { left, right, .. } => {
                left.find_pane(id).or_else(|| right.find_pane(id))
            }
            SplitNode::Vertical { top, bottom, .. } => {
                top.find_pane(id).or_else(|| bottom.find_pane(id))
            }
            _ => None,
        }
    }

    /// 关闭指定ID的面板，返回是否成功关闭
    /// 如果关闭的是唯一的面板，返回 false
    /// 如果成功关闭，用兄弟节点替换父节点，返回 true
    pub fn close_pane(&mut self, id: usize) -> bool {
        match self {
            SplitNode::Leaf { id: leaf_id, .. } if *leaf_id == id => {
                // 不能关闭唯一的面板
                false
            }
            SplitNode::Horizontal { left, right, .. } => {
                // 检查左侧是否包含要关闭的面板
                if let SplitNode::Leaf { id: left_id, .. } = left.as_ref() {
                    if *left_id == id {
                        // 关闭左侧，用右侧替换整个节点
                        *self = *right.clone();
                        return true;
                    }
                }
                // 检查右侧是否包含要关闭的面板
                if let SplitNode::Leaf { id: right_id, .. } = right.as_ref() {
                    if *right_id == id {
                        // 关闭右侧，用左侧替换整个节点
                        *self = *left.clone();
                        return true;
                    }
                }
                // 递归查找
                left.close_pane(id) || right.close_pane(id)
            }
            SplitNode::Vertical { top, bottom, .. } => {
                // 检查上方是否包含要关闭的面板
                if let SplitNode::Leaf { id: top_id, .. } = top.as_ref() {
                    if *top_id == id {
                        // 关闭上方，用下方替换整个节点
                        *self = *bottom.clone();
                        return true;
                    }
                }
                // 检查下方是否包含要关闭的面板
                if let SplitNode::Leaf { id: bottom_id, .. } = bottom.as_ref() {
                    if *bottom_id == id {
                        // 关闭下方，用上方替换整个节点
                        *self = *top.clone();
                        return true;
                    }
                }
                // 递归查找
                top.close_pane(id) || bottom.close_pane(id)
            }
            _ => false,
        }
    }

    /// 获取所有叶子节点的ID列表
    pub fn collect_pane_ids(&self) -> Vec<usize> {
        match self {
            SplitNode::Leaf { id, .. } => vec![*id],
            SplitNode::Horizontal { left, right, .. } => {
                let mut ids = left.collect_pane_ids();
                ids.extend(right.collect_pane_ids());
                ids
            }
            SplitNode::Vertical { top, bottom, .. } => {
                let mut ids = top.collect_pane_ids();
                ids.extend(bottom.collect_pane_ids());
                ids
            }
        }
    }

    /// 查找当前面板在指定方向上的相邻面板
    /// 返回相邻面板的ID，如果没有则返回None
    pub fn find_adjacent_pane(&self, current_id: usize, direction: Direction) -> Option<usize> {
        self.find_adjacent_internal(current_id, direction, true)
    }

    /// 内部递归查找相邻面板
    fn find_adjacent_internal(&self, current_id: usize, direction: Direction, is_root: bool) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => {
                if *id == current_id {
                    // 找到了当前面板，但在叶子节点层面没有相邻的
                    None
                } else if is_root {
                    // 如果这是根节点且是叶子，说明只有一个面板
                    None
                } else {
                    // 返回这个面板作为候选
                    Some(*id)
                }
            }
            SplitNode::Horizontal { left, right, .. } => {
                match direction {
                    Direction::Left => {
                        // 如果当前面板在右侧，返回左侧的最右面板
                        if right.contains_pane(current_id) {
                            left.get_rightmost_pane()
                        } else {
                            // 当前面板在左侧，继续向上查找
                            left.find_adjacent_internal(current_id, direction, false)
                        }
                    }
                    Direction::Right => {
                        // 如果当前面板在左侧，返回右侧的最左面板
                        if left.contains_pane(current_id) {
                            right.get_leftmost_pane()
                        } else {
                            // 当前面板在右侧，继续向上查找
                            right.find_adjacent_internal(current_id, direction, false)
                        }
                    }
                    Direction::Up | Direction::Down => {
                        // 水平分割不影响上下导航
                        left.find_adjacent_internal(current_id, direction, false)
                            .or_else(|| right.find_adjacent_internal(current_id, direction, false))
                    }
                }
            }
            SplitNode::Vertical { top, bottom, .. } => {
                match direction {
                    Direction::Up => {
                        // 如果当前面板在下方，返回上方的最下面板
                        if bottom.contains_pane(current_id) {
                            top.get_bottommost_pane()
                        } else {
                            // 当前面板在上方，继续向上查找
                            top.find_adjacent_internal(current_id, direction, false)
                        }
                    }
                    Direction::Down => {
                        // 如果当前面板在上方，返回下方的最上面板
                        if top.contains_pane(current_id) {
                            bottom.get_topmost_pane()
                        } else {
                            // 当前面板在下方，继续向上查找
                            bottom.find_adjacent_internal(current_id, direction, false)
                        }
                    }
                    Direction::Left | Direction::Right => {
                        // 垂直分割不影响左右导航
                        top.find_adjacent_internal(current_id, direction, false)
                            .or_else(|| bottom.find_adjacent_internal(current_id, direction, false))
                    }
                }
            }
        }
    }

    /// 检查是否包含指定ID的面板
    fn contains_pane(&self, id: usize) -> bool {
        match self {
            SplitNode::Leaf { id: leaf_id, .. } => *leaf_id == id,
            SplitNode::Horizontal { left, right, .. } => {
                left.contains_pane(id) || right.contains_pane(id)
            }
            SplitNode::Vertical { top, bottom, .. } => {
                top.contains_pane(id) || bottom.contains_pane(id)
            }
        }
    }

    /// 获取最左边的面板ID
    fn get_leftmost_pane(&self) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => Some(*id),
            SplitNode::Horizontal { left, .. } => left.get_leftmost_pane(),
            SplitNode::Vertical { top, .. } => top.get_leftmost_pane(),
        }
    }

    /// 获取最右边的面板ID
    fn get_rightmost_pane(&self) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => Some(*id),
            SplitNode::Horizontal { right, .. } => right.get_rightmost_pane(),
            SplitNode::Vertical { top, .. } => top.get_rightmost_pane(),
        }
    }

    /// 获取最上面的面板ID
    fn get_topmost_pane(&self) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => Some(*id),
            SplitNode::Horizontal { left, .. } => left.get_topmost_pane(),
            SplitNode::Vertical { top, .. } => top.get_topmost_pane(),
        }
    }

    /// 获取最下面的面板ID
    fn get_bottommost_pane(&self) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => Some(*id),
            SplitNode::Horizontal { left, .. } => left.get_bottommost_pane(),
            SplitNode::Vertical { bottom, .. } => bottom.get_bottommost_pane(),
        }
    }
}

/// 导航方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
