use serde::{Deserialize, Serialize};

/// 路径步骤 - 记录在父节点中的位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathStep {
    Left,
    Right,
    Top,
    Bottom,
}

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
        // 1. 查找从根到当前节点的路径
        let mut path = Vec::new();
        if !self.find_node_path(current_id, &mut path) {
            return None; // 节点不存在
        }

        // 2. 从路径末尾向上遍历，查找匹配的父容器
        for i in (0..path.len()).rev() {
            let step = path[i];

            // 获取父节点（通过路径重新遍历）
            let parent_node = self.get_node_at_path(&path[..i]);

            // 3. 检查父节点类型是否匹配方向，并获取兄弟节点
            let sibling_node = match (parent_node, direction, step) {
                // 水平容器 + 左右方向
                (Some(SplitNode::Horizontal { left, right, .. }), Direction::Left, PathStep::Right) => {
                    Some(left.as_ref())
                }
                (Some(SplitNode::Horizontal { left, right, .. }), Direction::Right, PathStep::Left) => {
                    Some(right.as_ref())
                }
                // 垂直容器 + 上下方向
                (Some(SplitNode::Vertical { top, bottom, .. }), Direction::Up, PathStep::Bottom) => {
                    Some(top.as_ref())
                }
                (Some(SplitNode::Vertical { top, bottom, .. }), Direction::Down, PathStep::Top) => {
                    Some(bottom.as_ref())
                }
                _ => None,
            };

            // 如果找到兄弟节点，根据当前路径选择位置感知的叶子
            if let Some(sibling) = sibling_node {
                return sibling.get_leaf_with_preference(&path);
            }
        }

        None // 到达根节点仍无匹配
    }

    /// 根据路径获取节点引用
    fn get_node_at_path(&self, path: &[PathStep]) -> Option<&SplitNode> {
        let mut current = self;
        for &step in path {
            current = match (current, step) {
                (SplitNode::Horizontal { left, .. }, PathStep::Left) => left,
                (SplitNode::Horizontal { right, .. }, PathStep::Right) => right,
                (SplitNode::Vertical { top, .. }, PathStep::Top) => top,
                (SplitNode::Vertical { bottom, .. }, PathStep::Bottom) => bottom,
                _ => return None,
            };
        }
        Some(current)
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

    /// 查找从根到目标节点的路径
    fn find_node_path(&self, target_id: usize, path: &mut Vec<PathStep>) -> bool {
        match self {
            SplitNode::Leaf { id, .. } => *id == target_id,
            SplitNode::Horizontal { left, right, .. } => {
                path.push(PathStep::Left);
                if left.find_node_path(target_id, path) {
                    return true;
                }
                path.pop();

                path.push(PathStep::Right);
                if right.find_node_path(target_id, path) {
                    return true;
                }
                path.pop();
                false
            }
            SplitNode::Vertical { top, bottom, .. } => {
                path.push(PathStep::Top);
                if top.find_node_path(target_id, path) {
                    return true;
                }
                path.pop();

                path.push(PathStep::Bottom);
                if bottom.find_node_path(target_id, path) {
                    return true;
                }
                path.pop();
                false
            }
        }
    }

    /// 获取分支的第一个 Leaf ID
    fn get_first_leaf(&self) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => Some(*id),
            SplitNode::Horizontal { left, .. } => left.get_first_leaf(),
            SplitNode::Vertical { top, .. } => top.get_first_leaf(),
        }
    }

    /// 根据路径偏好获取叶子节点
    /// 保持位置感知：向上/下导航时保持左/右位置，向左/右导航时保持上/下位置
    fn get_leaf_with_preference(&self, path: &[PathStep]) -> Option<usize> {
        match self {
            SplitNode::Leaf { id, .. } => Some(*id),
            SplitNode::Horizontal { left, right, .. } => {
                // 检查路径中是否有Left/Right偏好
                let prefer_right = path.iter().any(|&step| step == PathStep::Right);
                if prefer_right {
                    right.get_leaf_with_preference(path)
                } else {
                    left.get_leaf_with_preference(path)
                }
            }
            SplitNode::Vertical { top, bottom, .. } => {
                // 检查路径中是否有Top/Bottom偏好
                let prefer_bottom = path.iter().any(|&step| step == PathStep::Bottom);
                if prefer_bottom {
                    bottom.get_leaf_with_preference(path)
                } else {
                    top.get_leaf_with_preference(path)
                }
            }
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
