use crate::app::GotoTarget;

const LABEL_ALPHABET: &str = "asdfghjklqwertyuiopzxcvbnm";

pub fn labels(count: usize) -> Vec<String> {
    let alphabet: Vec<char> = LABEL_ALPHABET.chars().collect();
    let radix = alphabet.len();

    (0..count)
        .map(|index| {
            let first = alphabet[(index / radix) % radix];
            let second = alphabet[index % radix];
            format!("{first}{second}")
        })
        .collect()
}

pub fn label_for_task(
    state: &crate::app::GotoState,
    pane_id: usize,
    column: usize,
    task_index: usize,
) -> Option<&str> {
    state.labels.iter().find_map(|(label, target)| {
        if *target
            == (GotoTarget::Task {
                pane_id,
                column,
                task_index,
            })
        {
            Some(label.as_str())
        } else {
            None
        }
    })
}

pub fn label_for_project(state: &crate::app::GotoState, item_index: usize) -> Option<&str> {
    state.labels.iter().find_map(|(label, target)| {
        if *target == (GotoTarget::Project { item_index }) {
            Some(label.as_str())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::labels;

    #[test]
    fn creates_unique_two_character_labels_in_display_order() {
        let labels = labels(30);

        assert_eq!(labels[0], "aa");
        assert_eq!(labels[25], "am");
        assert_eq!(labels[26], "sa");
        assert_eq!(
            labels.iter().map(String::len).collect::<Vec<_>>(),
            vec![2; 30]
        );
        assert_eq!(
            labels
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            labels.len()
        );
    }
}
