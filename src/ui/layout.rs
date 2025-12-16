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
        self.find_adjacent_with_position(current_id, direction)
    }

    /// 使用位置感知的方式查找相邻面板
    fn find_adjacent_with_position(&self, current_id: usize, direction: Direction) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => {
                if *id == current_id {
                    None
                } else {
                    None
                }
            }
            SplitNode::Horizontal { left, right, .. } => {
                let in_left = left.contains_pane(current_id);
                let in_right = right.contains_pane(current_id);

                match direction {
                    Direction::Left => {
                        if in_right {
                            // 从右边往左找：找到左边对应位置的面板
                            left.find_adjacent_with_position(current_id, direction)
                                .or_else(|| right.get_corresponding_pane(left, current_id, true))
                        } else {
                            // 在左边继续向左找
                            left.find_adjacent_with_position(current_id, direction)
                        }
                    }
                    Direction::Right => {
                        if in_left {
                            // 从左边往右找：找到右边对应位置的面板
                            right.find_adjacent_with_position(current_id, direction)
                                .or_else(|| left.get_corresponding_pane(right, current_id, false))
                        } else {
                            // 在右边继续向右找
                            right.find_adjacent_with_position(current_id, direction)
                        }
                    }
                    Direction::Up | Direction::Down => {
                        // 在水平分割中，上下移动需要穿透到子节点
                        if in_left {
                            left.find_adjacent_with_position(current_id, direction)
                        } else {
                            right.find_adjacent_with_position(current_id, direction)
                        }
                    }
                }
            }
            SplitNode::Vertical { top, bottom, .. } => {
                let in_top = top.contains_pane(current_id);
                let in_bottom = bottom.contains_pane(current_id);

                match direction {
                    Direction::Up => {
                        if in_bottom {
                            // 从下往上找：找到上边对应位置的面板
                            top.find_adjacent_with_position(current_id, direction)
                                .or_else(|| bottom.get_corresponding_pane(top, current_id, true))
                        } else {
                            // 在上边继续向上找
                            top.find_adjacent_with_position(current_id, direction)
                        }
                    }
                    Direction::Down => {
                        if in_top {
                            // 从上往下找：找到下边对应位置的面板
                            bottom.find_adjacent_with_position(current_id, direction)
                                .or_else(|| top.get_corresponding_pane(bottom, current_id, false))
                        } else {
                            // 在下边继续向下找
                            bottom.find_adjacent_with_position(current_id, direction)
                        }
                    }
                    Direction::Left | Direction::Right => {
                        // 在垂直分割中，左右移动需要穿透到子节点
                        if in_top {
                            top.find_adjacent_with_position(current_id, direction)
                        } else {
                            bottom.find_adjacent_with_position(current_id, direction)
                        }
                    }
                }
            }
        }
    }

    /// 在目标子树中找到与当前面板对应位置的面板
    /// from_left: true 表示从左/上往右/下找，false 表示从右/下往左/上找
    fn get_corresponding_pane(&self, target: &SplitNode, current_id: usize, from_left: bool) -> Option<usize> {
        // 判断当前面板在本子树中的位置（左/右 或 上/下）
        match self {
            SplitNode::Leaf { .. } => {
                // 叶子节点直接返回目标的首个面板
                if from_left {
                    target.get_leftmost_pane()
                } else {
                    target.get_rightmost_pane()
                }
            }
            SplitNode::Horizontal { left, right, .. } => {
                if left.contains_pane(current_id) {
                    // 当前在左边，找目标的左边
                    target.get_leftmost_pane()
                } else {
                    // 当前在右边，找目标的右边
                    target.get_rightmost_pane()
                }
            }
            SplitNode::Vertical { top, bottom, .. } => {
                if top.contains_pane(current_id) {
                    // 当前在上边，找目标的上边
                    target.get_topmost_pane()
                } else {
                    // 当前在下边，找目标的下边
                    target.get_bottommost_pane()
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
