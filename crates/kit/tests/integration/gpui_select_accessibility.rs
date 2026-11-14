//! The Windows provider derives ExpandCollapsePattern from `aria_expanded`.
//! Its actions must be registered on the same semantic control, not only Click.
use gpui_kit::{
    Element as _, IntoElement as _, RenderOnce as _, Role, TestAppContext, accesskit, base::Select,
};

#[gpui_kit::test]
fn select_registers_expand_collapse_except_when_disabled(cx: &mut TestAppContext) {
    let window = cx.add_empty_window();
    window.update(|window, cx| {
        for open in [false, true] {
            for disabled in [false, true] {
                let mut node = accesskit::Node::new(Role::ComboBox);
                Select::new("language")
                    .open(open)
                    .disabled(disabled)
                    .render(window, cx)
                    .into_element()
                    .write_a11y_info(&mut node);

                assert_eq!(node.is_expanded(), Some(open));
                for action in [
                    accesskit::Action::Click,
                    accesskit::Action::Expand,
                    accesskit::Action::Collapse,
                ] {
                    assert_eq!(
                        node.supports_action(action),
                        !disabled,
                        "{action:?}: open={open}, disabled={disabled}"
                    );
                }
            }
        }
    });
}
