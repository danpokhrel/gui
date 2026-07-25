use dear_imnodes::{self as imnodes, ColorElement};

/// A cohesive dark theme for our node editor.
pub struct EditorTheme {
    pub title_bar: [f32; 4],
    pub title_hovered: [f32; 4],
    pub title_selected: [f32; 4],
    pub node_bg: [f32; 4],
    pub node_border: [f32; 4],
    pub link: [f32; 4],
    pub link_hovered: [f32; 4],
    pub link_selected: [f32; 4],
    pub grid_bg: [f32; 4],
    pub grid_line: [f32; 4],
    pub grid_line_pri: [f32; 4],
    pub grid_spacing: f32,
    pub node_rounding: f32,
    pub node_padding: [f32; 2],
    pub link_thickness: f32,
    pub pin_radius: f32,
}

impl EditorTheme {
    /// A modern dark theme inspired by Blender's node editor.
    pub fn dark() -> Self {
        Self {
            title_bar: [0.15, 0.29, 0.52, 1.0],
            title_hovered: [0.20, 0.36, 0.60, 1.0],
            title_selected: [0.25, 0.44, 0.72, 1.0],
            node_bg: [0.12, 0.13, 0.16, 1.0],
            node_border: [0.25, 0.26, 0.30, 1.0],
            link: [0.55, 0.75, 0.95, 1.0],
            link_hovered: [0.75, 0.90, 1.00, 1.0],
            link_selected: [0.95, 0.80, 0.40, 1.0],
            grid_bg: [0.06, 0.07, 0.08, 1.0],
            grid_line: [0.15, 0.16, 0.18, 1.0],
            grid_line_pri: [0.22, 0.23, 0.26, 1.0],
            grid_spacing: 24.0,
            node_rounding: 4.0,
            node_padding: [8.0, 8.0],
            link_thickness: 3.0,
            pin_radius: 6.0,
        }
    }

    /// Apply all colors and style variables to an editor frame.
    pub fn apply(&self, editor: &imnodes::NodeEditor<'_>) {
        editor.set_color(ColorElement::TitleBar, self.title_bar);
        editor.set_color(ColorElement::TitleBarHovered, self.title_hovered);
        editor.set_color(ColorElement::TitleBarSelected, self.title_selected);
        editor.set_color(ColorElement::NodeBackground, self.node_bg);
        // `ColorElement::NodeBorder` does not exist in dear-imnodes 0.15.1;
        // the correct variant is `NodeOutline`. (The field is named `node_border`
        // for readability; the tutorial uses the same `NodeOutline` variant.)
        editor.set_color(ColorElement::NodeOutline, self.node_border);
        editor.set_color(ColorElement::Link, self.link);
        editor.set_color(ColorElement::LinkHovered, self.link_hovered);
        editor.set_color(ColorElement::LinkSelected, self.link_selected);
        editor.set_color(ColorElement::GridBackground, self.grid_bg);
        editor.set_color(ColorElement::GridLine, self.grid_line);
        editor.set_color(ColorElement::GridLinePrimary, self.grid_line_pri);

        editor.set_grid_spacing(self.grid_spacing);
        editor.set_node_corner_rounding(self.node_rounding);
        editor.set_node_padding(self.node_padding);
        editor.set_link_thickness(self.link_thickness);
        editor.set_pin_circle_radius(self.pin_radius);
    }
}
