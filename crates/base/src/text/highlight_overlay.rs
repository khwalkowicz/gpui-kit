use std::{collections::BTreeSet, ops::Range};

pub(super) fn overlay_highlights<T: Default>(
    base: impl IntoIterator<Item = (Range<usize>, T)>,
    overlay: impl IntoIterator<Item = (Range<usize>, T)>,
    merge: impl Fn(T, &T) -> T,
) -> Vec<(Range<usize>, T)> {
    let mut endpoints = Vec::new();
    let mut highlights = Vec::new();
    for (range, highlight) in base.into_iter().chain(overlay) {
        if !range.is_empty() {
            let id = highlights.len();
            endpoints.push((range.start, id, true));
            endpoints.push((range.end, id, false));
            highlights.push(highlight);
        }
    }
    endpoints.sort_unstable();
    let mut active = BTreeSet::new();
    let mut combined = Vec::new();
    let mut cursor = 0;
    for (position, id, start) in endpoints {
        if position > cursor && !active.is_empty() {
            let highlight = active.iter().fold(T::default(), |combined, id: &usize| {
                merge(combined, &highlights[*id])
            });
            combined.push((cursor..position, highlight));
        }
        cursor = position;
        if start {
            active.insert(id);
        } else {
            active.remove(&id);
        }
    }
    combined
}

#[cfg(test)]
mod tests {
    use super::overlay_highlights;

    #[derive(Debug, Default, PartialEq)]
    struct Appearance {
        color: Option<u8>,
        bold: bool,
        italic: bool,
    }

    fn merge(base: Appearance, overlay: &Appearance) -> Appearance {
        Appearance {
            color: overlay.color.or(base.color),
            bold: base.bold || overlay.bold,
            italic: base.italic || overlay.italic,
        }
    }

    #[test]
    fn whole_link_overlay_wins_when_formatted_runs_start_later() {
        for color in [2, 3, 4] {
            let base = [
                (
                    0..3,
                    Appearance {
                        color: Some(1),
                        ..Default::default()
                    },
                ),
                (
                    3..7,
                    Appearance {
                        color: Some(1),
                        bold: true,
                        ..Default::default()
                    },
                ),
                (
                    7..10,
                    Appearance {
                        color: Some(1),
                        italic: true,
                        ..Default::default()
                    },
                ),
            ];
            let result = overlay_highlights(
                base,
                [(
                    0..10,
                    Appearance {
                        color: Some(color),
                        ..Default::default()
                    },
                )],
                merge,
            );
            assert_eq!(result.len(), 3);
            assert!(
                result
                    .iter()
                    .all(|(_, appearance)| appearance.color == Some(color))
            );
            assert!(result[1].1.bold);
            assert!(result[2].1.italic);
        }
    }

    #[test]
    fn overlay_boundaries_preserve_plain_gaps_and_separate_adjacent_links() {
        let result = overlay_highlights(
            [(
                0..12,
                Appearance {
                    color: Some(1),
                    ..Default::default()
                },
            )],
            [
                (
                    2..5,
                    Appearance {
                        color: Some(2),
                        ..Default::default()
                    },
                ),
                (
                    5..8,
                    Appearance {
                        color: Some(3),
                        ..Default::default()
                    },
                ),
            ],
            merge,
        );
        assert_eq!(
            result
                .iter()
                .map(|(range, appearance)| (range.clone(), appearance.color))
                .collect::<Vec<_>>(),
            [
                (0..2, Some(1)),
                (2..5, Some(2)),
                (5..8, Some(3)),
                (8..12, Some(1))
            ]
        );
    }
}
