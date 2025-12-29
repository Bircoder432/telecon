use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::RwLock;

use crate::{
    app::service_manager::ServiceManager,
    bot::sender::TelegramSender,
    domain::{action::Action, notification::Notification},
    parser::{HandlerRegistry, tree::Node},
};

pub struct ActionDispatcher {
    sender: TelegramSender,
    service_manager: ServiceManager,
    services: Arc<RwLock<Node>>,
    handlers: Arc<RwLock<HandlerRegistry>>,
}

impl ActionDispatcher {
    pub fn new(
        bot: Bot,
        chat_id: ChatId,
        service_manager: ServiceManager,
        services: Arc<RwLock<Node>>,
        handlers: Arc<RwLock<HandlerRegistry>>,
    ) -> Self {
        Self {
            sender: TelegramSender::new(bot, chat_id),
            service_manager,
            services,
            handlers,
        }
    }

    pub async fn dispatch(&self, action: Action) {
        match action {
            Action::Notify(n) => {
                self.sender.send(n).await;
            }
            Action::ReloadServices => match self.service_manager.reload().await {
                Ok(_) => {
                    self.sender
                        .send(Notification {
                            text: Some("Services reloaded".into()),
                            files: vec![],
                            media: vec![],
                            buttons: vec![],
                        })
                        .await;
                }
                Err(e) => {
                    self.sender
                        .send(Notification {
                            text: Some(format!("Reload error: {e}")),
                            files: vec![],
                            media: vec![],
                            buttons: vec![],
                        })
                        .await;
                }
            },
        }
    }

    async fn reload_services(&self) {
        // ПОКА сюда просто перенесём код,
        // позже вынесем ещё дальше
        println!("Reload services requested");
    }
}
