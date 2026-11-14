//! Minimal reproducer for caption foreground/background disagreement. It uses
//! the same inherited-color path as GPUI Kit's Icon, without native windows.
#![allow(clippy::expect_used, reason = "test-owned windows and fixed keystroke")]
use std::{cell::Cell, rc::Rc};

use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Context, Hsla, InteractiveElement as _, IntoElement,
    KeyDownEvent, Keystroke, ParentElement as _, PlatformInput, Render, RenderOnce, Styled as _,
    TestAppContext, Window, black, div, point, px, red, white,
};

#[derive(IntoElement)]
struct InheritedColor(Rc<Cell<Hsla>>);

impl RenderOnce for InheritedColor {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.0.set(window.text_style().color);
        div()
    }
}

#[derive(Clone, Copy)]
enum HoverTarget {
    Element,
    Group,
}

struct CaptionProbe {
    color: Rc<Cell<Hsla>>,
    target: HoverTarget,
}

impl Render for CaptionProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let caption = div()
            .id("caption-close")
            .size(px(60.0))
            .bg(white())
            .text_color(black())
            .child(InheritedColor(self.color.clone()));
        let caption = match self.target {
            HoverTarget::Element => caption.hover(|style| style.bg(red()).text_color(white())),
            HoverTarget::Group => {
                caption.group_hover("caption", |style| style.bg(red()).text_color(white()))
            }
        };
        div().size_full().child(
            div()
                .m(px(20.0))
                .size(px(60.0))
                .group("caption")
                .child(caption),
        )
    }
}

#[gpui_kit::test]
fn keyboard_modality_clears_inherited_hover_foreground(cx: &mut TestAppContext) {
    check_keyboard_transition(cx, HoverTarget::Element);
}

#[gpui_kit::test]
fn keyboard_modality_clears_inherited_group_hover_foreground(cx: &mut TestAppContext) {
    check_keyboard_transition(cx, HoverTarget::Group);
}

fn hovered_caption(
    cx: &mut TestAppContext,
    target: HoverTarget,
) -> (AnyWindowHandle, Rc<Cell<Hsla>>) {
    let color = Rc::new(Cell::new(red()));
    let window: AnyWindowHandle = cx
        .add_window({
            let color = color.clone();
            move |_, _| CaptionProbe { color, target }
        })
        .into();
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .expect("fixture window exists");
    assert_eq!(color.get(), black());
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(30.0), px(30.0)), cx);
    })
    .expect("fixture window exists");
    assert_eq!(color.get(), white());
    (window, color)
}

fn check_keyboard_transition(cx: &mut TestAppContext, target: HoverTarget) {
    let (window, color) = hovered_caption(cx, target);
    cx.update_window(window, |_, window, cx| {
        window.dispatch_event(
            PlatformInput::KeyDown(KeyDownEvent {
                keystroke: Keystroke::parse("tab").expect("fixed Tab key parses"),
                is_held: false,
                prefer_character_input: false,
            }),
            cx,
        );
        assert!(window.last_input_was_keyboard());
        window.draw(cx).clear(cx);
    })
    .expect("fixture window exists");
    assert_eq!(
        color.get(),
        black(),
        "Child foreground must leave hover with its parent background"
    );
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(40.0), px(40.0)), cx);
    })
    .expect("fixture window exists");
    assert_eq!(color.get(), white(), "Pointer input restores hover");
    cx.update_window(window, |_, window, cx| {
        window.simulate_mouse_move(point(px(120.0), px(120.0)), cx);
    })
    .expect("fixture window exists");
    assert_eq!(color.get(), black(), "Pointer exit clears hover");
}

#[gpui_kit::test]
fn bounds_refresh_clears_inherited_hover_without_mouse_move(cx: &mut TestAppContext) {
    let (window, color) = hovered_caption(cx, HoverTarget::Element);
    cx.update_window(window, |_, window, cx| {
        // Activation/move/resize refresh the position directly from the platform,
        // without dispatching MouseMove. The headless platform reports (0, 0),
        // outside this caption, just as reopening under an outside pointer does.
        window.bounds_changed(cx);
        assert_eq!(window.mouse_position(), point(px(0.0), px(0.0)));
        window.draw(cx).clear(cx);
    })
    .expect("fixture window exists");
    assert_eq!(
        color.get(),
        black(),
        "Foreground must reflect the refreshed pointer position"
    );
}
