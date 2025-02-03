macro_rules! byte_image {
    (
        $ui:expr,
        $icon:literal
    ) => {
        let tex = $ui.ctx().load_texture(
            "icon_box",
            include_image!($icon),
            Default::default(),
        );
    };
}