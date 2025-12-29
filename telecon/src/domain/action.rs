use crate::domain::notification::Notification;

#[derive(Debug, Clone)]
pub enum Action {
    Notify(Notification),
    ReloadServices,
}
