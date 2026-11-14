//! Inert controls must also report disabled to assistive technology.
use gpui_kit::{
    Context, Element as _, IntoElement, Render, RenderOnce as _, Role, TestAppContext, Window,
    accesskit,
    base::{Button, Checkbox, Radio, Select, Switch},
    canvas,
};

fn assert_disabled_state(element: impl IntoElement, disabled: bool) {
    let mut node = accesskit::Node::new(Role::Unknown);
    element.into_element().write_a11y_info(&mut node);
    assert_eq!(node.is_disabled(), disabled);
    if disabled {
        assert!(!node.supports_action(accesskit::Action::Click));
    }
}

struct ControlProbe;

impl Render for ControlProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // The controls acquire their retained focus handles during layout/paint.
        canvas(
            |_, window, cx| {
                for disabled in [false, true, false] {
                    let mut node = accesskit::Node::new(Role::Button);
                    Button::new("export-history")
                        .accessibility_label("Export history")
                        .disabled(disabled)
                        .on_click(|_, _, _| {})
                        .render(window, cx)
                        .into_element()
                        .write_a11y_info(&mut node);

                    assert_eq!(node.label(), Some("Export history"));
                    assert_eq!(node.is_disabled(), disabled);
                    assert_eq!(node.supports_action(accesskit::Action::Click), !disabled);
                    assert_disabled_state(
                        Checkbox::new("checkbox")
                            .disabled(disabled)
                            .render(window, cx),
                        disabled,
                    );
                    assert_disabled_state(
                        Radio::new("radio").disabled(disabled).render(window, cx),
                        disabled,
                    );
                    assert_disabled_state(
                        Switch::new("switch").disabled(disabled).render(window, cx),
                        disabled,
                    );
                    assert_disabled_state(
                        Select::new("select").disabled(disabled).render(window, cx),
                        disabled,
                    );
                }
            },
            |_, _, _, _| {},
        )
    }
}

#[gpui_kit::test]
fn controls_expose_disabled_state_without_actions(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| ControlProbe);
    cx.update(|window, cx| window.draw(cx).clear(cx));
}
