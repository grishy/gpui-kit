use gpui::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, relative,
};

use crate::{ActiveTheme as _, StyledExt as _, dialog::Confirm, h_flex};

/// Footer section of a dialog, typically contains action buttons.
///
/// # Examples
///
/// ```ignore
/// DialogFooter::new()
///     .child(DialogClose::new().child(Button::new("cancel").label("Cancel")))
///     .child(Button::new("confirm").label("Confirm"))
/// ```
#[derive(IntoElement)]
pub struct DialogFooter {
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl DialogFooter {
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            children: Vec::new(),
        }
    }
}

impl ParentElement for DialogFooter {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for DialogFooter {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for DialogFooter {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .gap_2()
            .justify_end()
            .line_height(relative(1.))
            .rounded_b(cx.theme().radius_lg)
            .refine_style(&self.style)
            .children(self.children)
    }
}

pub trait DialogFooterButton {
    fn is_cancel(&self) -> bool {
        false
    }

    fn is_action(&self) -> bool {
        false
    }
}
#[derive(IntoElement)]
pub struct DialogClose {
    base: gpui_base::DialogClose,
}

impl DialogClose {
    pub fn new() -> Self {
        Self {
            base: gpui_base::DialogClose::new(),
        }
    }
}

impl ParentElement for DialogClose {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl RenderOnce for DialogClose {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().size_full().child(self.base)
    }
}

#[derive(IntoElement)]
pub struct DialogAction {
    children: Vec<AnyElement>,
}

impl DialogAction {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ParentElement for DialogAction {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for DialogAction {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .size_full()
            .id("dialog-action")
            .on_click(move |_, window, cx| {
                window.dispatch_action(Box::new(Confirm { secondary: false }), cx)
            })
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Disableable as _, button::Button, dialog::Cancel};
    use gpui::{Context, FocusHandle, KeyDownEvent, KeyUpEvent, Keystroke, Render, point, px};
    use std::{cell::Cell, rc::Rc};

    #[gpui::test]
    fn close_child_activates_once_and_ignores_loading_and_disabled(cx: &mut gpui::TestAppContext) {
        struct Harness {
            focus: FocusHandle,
            cancels: Rc<Cell<usize>>,
            loading: bool,
            disabled: bool,
        }
        impl Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let cancels = self.cancels.clone();
                div()
                    .id("host")
                    .track_focus(&self.focus.clone().tab_stop(false))
                    .tab_group()
                    .on_action(move |_: &Cancel, _, _| cancels.set(cancels.get() + 1))
                    .child(
                        DialogClose::new().child(
                            Button::new("close")
                                .size(px(100.))
                                .loading(self.loading)
                                .disabled(self.disabled),
                        ),
                    )
            }
        }

        cx.update(crate::init);
        let cancels = Rc::new(Cell::new(0));
        let (view, cx) = cx.add_window_view({
            let cancels = cancels.clone();
            move |_, cx| Harness {
                focus: cx.focus_handle(),
                cancels,
                loading: false,
                disabled: false,
            }
        });
        cx.update(|window, cx| {
            view.read(cx).focus.clone().focus(window, cx);
            window.draw(cx).clear(cx);
        });
        cx.simulate_click(point(px(10.), px(10.)), Default::default());
        cx.run_until_parked();
        assert_eq!(cancels.get(), 1);
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.draw(cx).clear(cx);
        });
        let press_space = |cx: &mut gpui::VisualTestContext| {
            let keystroke = Keystroke::parse("space").unwrap();
            cx.simulate_event(KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(KeyUpEvent { keystroke });
        };
        press_space(cx);
        cx.run_until_parked();
        assert_eq!(cancels.get(), 2);

        for loading in [true, false] {
            view.update(cx, |view, cx| {
                view.loading = loading;
                view.disabled = !loading;
                cx.notify();
            });
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_click(point(px(10.), px(10.)), Default::default());
            press_space(cx);
            cx.run_until_parked();
            assert_eq!(cancels.get(), 2, "inactive close buttons must not cancel");
        }
    }
}
