use crate::client::redux::state::State;
use crate::client::view::app::chats_container::chats::{
    build_chat_view_module, ChatView, ChatViewModule,
};
use crate::client::view::app::chats_container::messages::{
    build_messages_view_module, MessagesView, MessagesViewModule,
};
use crate::client::view::app::chats_container::typing::{
    build_typing_view_module, TypingView, TypingViewModule,
};
use crate::client::view::View;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use shaku::{module, Component, Interface};
use std::sync::Arc;

mod chats;
mod messages;
mod typing;

pub trait ChatContainerView: View + Interface {}

#[derive(Component)]
#[shaku(interface = ChatContainerView)]
pub struct ChatContainerViewImpl {
    #[shaku(inject)]
    chats: Arc<dyn ChatView>,
    #[shaku(inject)]
    messages: Arc<dyn MessagesView>,
    #[shaku(inject)]
    typing: Arc<dyn TypingView>,
}

impl ChatContainerView for ChatContainerViewImpl {}

impl View for ChatContainerViewImpl {
    fn draw(&self, f: &mut Frame, rect: Rect, state: State) -> anyhow::Result<()> {
        let chunks_main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(rect);

        let chats_chunk = chunks_main[0];

        let chunks_right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(chunks_main[1]);

        let messages_chunk = chunks_right[1];
        let typing_chunk = chunks_right[0];

        self.chats.draw(f, chats_chunk, state.clone())?;
        self.messages.draw(f, messages_chunk, state.clone())?;
        self.typing.draw(f, typing_chunk, state)?;

        Ok(())
    }
}

module! {
    pub ChatContainerViewModule {
        components = [ChatContainerViewImpl],
        providers = [],
        use ChatViewModule {
            components = [dyn ChatView],
            providers = [],
        },
        use MessagesViewModule {
            components = [dyn MessagesView],
            providers = [],
        },
        use TypingViewModule {
            components = [dyn TypingView],
            providers = [],
        },
    }
}
pub fn build_chat_container_view_module() -> Arc<ChatContainerViewModule> {
    Arc::new(
        ChatContainerViewModule::builder(
            build_chat_view_module(),
            build_messages_view_module(),
            build_typing_view_module(),
        )
        .build(),
    )
}
