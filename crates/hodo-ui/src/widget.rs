/// Wave-1 interaction state shared by every hodo-ui widget (§K.7, §K.9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WidgetState {
    pub hover: bool,
    pub focus: bool,
    pub pressed: bool,
    pub disabled: bool,
    pub selected: bool,
}

impl WidgetState {
    pub fn enabled(self) -> bool {
        !self.disabled
    }
}

/// The wave-1 widget set from K.8. Geometry/rendering arrive in later waves;
/// this enum fixes the names and roles the restyle agent programs against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Widget {
    IconButton,
    Button,
    TextField,
    TabStrip,
    Menu,
    Dialog,
    Snackbar,
    ProgressBar,
    Badge,
}

impl Widget {
    pub fn name(self) -> &'static str {
        match self {
            Widget::IconButton => "IconButton",
            Widget::Button => "Button",
            Widget::TextField => "TextField",
            Widget::TabStrip => "TabStrip",
            Widget::Menu => "Menu",
            Widget::Dialog => "Dialog",
            Widget::Snackbar => "Snackbar",
            Widget::ProgressBar => "ProgressBar",
            Widget::Badge => "Badge",
        }
    }
}

pub const WIDGETS: [Widget; 9] = [
    Widget::IconButton,
    Widget::Button,
    Widget::TextField,
    Widget::TabStrip,
    Widget::Menu,
    Widget::Dialog,
    Widget::Snackbar,
    Widget::ProgressBar,
    Widget::Badge,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_wave1_widgets_are_unique() {
        let mut names: Vec<_> = WIDGETS.iter().map(|w| w.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), WIDGETS.len());
    }

    #[test]
    fn disabled_wins_over_other_state() {
        let s = WidgetState {
            hover: true,
            disabled: true,
            ..Default::default()
        };
        assert!(!s.enabled());
    }
}
