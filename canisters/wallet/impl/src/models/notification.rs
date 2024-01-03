use super::{NotificationStatus, NotificationType, UserId};
use crate::errors::NotificationError;
use candid::{CandidType, Deserialize};
use ic_canister_core::{
    model::{ModelValidator, ModelValidatorResult},
    types::{Timestamp, UUID},
};
use ic_canister_macros::stable_object;

/// The notification id, which is a UUID.
pub type NotificationId = UUID;

/// Represents a notification within the system.
#[stable_object]
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Notification {
    pub id: NotificationId,
    pub notification_type: NotificationType,
    pub status: NotificationStatus,
    /// The user that the notification is targeted to.
    pub target_user_id: UserId,
    /// The title of the notification, which is a tuple of the title as the first
    /// entry in english and the second entry the locale key for the title.
    pub title: (String, String),
    /// The message of the notification, which is a tuple of the message as the first
    /// entry in english and the second entry the locale key for the message.
    pub message: (String, String),
    pub created_timestamp: Timestamp,
    pub last_modification_timestamp: Timestamp,
}

#[stable_object]
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NotificationKey {
    pub id: NotificationId,
}

fn validate_title(title: &(String, String)) -> ModelValidatorResult<NotificationError> {
    if title.0.len() > Notification::MAX_TITLE_LEN as usize {
        return Err(NotificationError::ValidationError {
            info: format!(
                "Notification title exceeds the maximum allowed: {}",
                Notification::MAX_TITLE_LEN
            ),
        });
    }

    Ok(())
}

fn validate_message(message: &(String, String)) -> ModelValidatorResult<NotificationError> {
    if message.0.len() > Notification::MAX_MESSAGE_LEN as usize {
        return Err(NotificationError::ValidationError {
            info: format!(
                "Notification message exceeds the maximum allowed: {}",
                Notification::MAX_MESSAGE_LEN
            ),
        });
    }

    Ok(())
}

impl ModelValidator<NotificationError> for Notification {
    fn validate(&self) -> ModelValidatorResult<NotificationError> {
        validate_title(&self.title)?;
        validate_message(&self.message)?;

        Ok(())
    }
}

impl Notification {
    pub const MAX_TITLE_LEN: u8 = 255;
    pub const MAX_MESSAGE_LEN: u32 = 4096;

    pub fn key(id: NotificationId) -> NotificationKey {
        NotificationKey { id }
    }

    pub fn to_key(&self) -> NotificationKey {
        Notification::key(self.id.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notification_test_utils::mock_notification;

    #[test]
    fn fail_notification_title_too_long() {
        let mut notitication = mock_notification();
        notitication.title.0 = "a".repeat(Notification::MAX_TITLE_LEN as usize + 1);

        let result = validate_title(&notitication.title);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            NotificationError::ValidationError {
                info: format!(
                    "Notification title exceeds the maximum allowed: {}",
                    Notification::MAX_TITLE_LEN
                )
            }
        );
    }

    #[test]
    fn fail_notification_message_too_long() {
        let mut notitication = mock_notification();
        notitication.message.0 = "a".repeat(Notification::MAX_MESSAGE_LEN as usize + 1);

        let result = validate_message(&notitication.message);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            NotificationError::ValidationError {
                info: format!(
                    "Notification message exceeds the maximum allowed: {}",
                    Notification::MAX_MESSAGE_LEN
                )
            }
        );
    }

    #[test]
    fn test_notification_validation() {
        let notitication = mock_notification();

        let result = notitication.validate();

        assert!(result.is_ok());
    }
}

#[cfg(test)]
pub mod notification_test_utils {
    use super::*;
    use crate::models::{NotificationStatus, NotificationType};

    pub fn mock_notification() -> Notification {
        Notification {
            id: [0; 16],
            status: NotificationStatus::Sent,
            target_user_id: [1; 16],
            message: ("message".to_string(), "message".to_string()),
            title: ("title".to_string(), "title".to_string()),
            notification_type: NotificationType::SystemMessage,
            created_timestamp: 0,
            last_modification_timestamp: 0,
        }
    }
}