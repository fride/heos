use std::cell::RefCell;
use std::rc::Rc;
use druid::{BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size, UnitPoint, UpdateCtx, Widget, WidgetExt};
use druid::im::OrdMap;
use druid::piet::PietTextLayoutBuilder;
use druid::text::TextStorage;
use druid::widget::{Button, CrossAxisAlignment, Flex, Label, List, Scroll, TextBox};
use im;
use crate::spielwiese::Ref;
use crate::spielwiese::state::Player;
use super::state::AppState;


fn ui_builder() -> impl Widget<AppState> {
    List::new(||{
        Label::new(|item: &(i64, Rc<RefCell<Player>>), _env: &_| format!("List item #{}", item.1.borrow().name))
            .align_vertical(UnitPoint::LEFT)
            .padding(10.0)
            .expand()
            .height(50.0)
            .background(Color::rgb(0.5, 0.5, 0.5))
    })
        .horizontal()
        .lens(AppState::players)
}
