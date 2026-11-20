//! Sends private and group messages and waits for the send result.
//!
//! [`BotContext::send_private_message`] and [`BotContext::send_group_message`] return a
//! [`SendMessageAsyncResponse`] handle; the message ID or an upstream error is only
//! available after calling one of its `wait_response` methods.

use crate::error::{Error, Result};
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use crate::prelude::MessageType;
use crate::prelude::SendMessage;
use crate::re_export::serde_json;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// An asynchronous response handle for a sent message.
///
/// When a send method returns this type, the request has been written to the current
/// WebSocket, but it is not yet confirmed whether the upstream accepted it. The handle must be
/// consumed before the result can be awaited; dropping the handle removes the local echo
/// registration and does not recall the message that was already sent.
pub struct SendMessageAsyncResponse(EchoAsyncResponse);

/// Data returned after a message is sent successfully.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct SendMessageResponse {
    /// Message ID assigned by the upstream, usable for recall, reply, or fetching message
    /// details.
    pub message_id: i64,
}

impl SendMessageAsyncResponse {
    /// Waits up to `timeout` for the send result and deserializes it into
    /// [`SendMessageResponse`].
    ///
    /// `timeout` only limits the response wait after the request has been sent; it does not
    /// include message serialization or the WebSocket write. This method consumes the handle,
    /// so it can only wait once.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when the response channel closes before a
    ///   response arrives.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within
    ///   `timeout`.
    /// - Returns [`crate::error::Error::StateError`] when the upstream `retcode` is non-zero.
    /// - Returns [`crate::error::Error::JsonError`] when the response `data` cannot be
    ///   deserialized into [`SendMessageResponse`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait; the message may already have been
    /// processed by the upstream.
    pub async fn wait_response_with_timeout(
        self,
        timeout: Duration,
    ) -> Result<SendMessageResponse> {
        Ok(serde_json::from_value(self.0.data(timeout).await?)?)
    }

    /// Calls [`Self::wait_response_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn wait_response(self) -> Result<SendMessageResponse> {
        self.wait_response_with_timeout(Duration::from_secs(10))
            .await
    }
}

impl BotContext {
    /// Sends a private message and returns a handle for waiting on the send result.
    ///
    /// A usable connection must already be established before calling this method. Returning
    /// `Ok` only means the request was written to the current WebSocket; it does not mean the
    /// message was delivered. Call [`SendMessageAsyncResponse::wait_response`] when the
    /// message ID is needed.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ ID of the recipient.
    /// - `message`: message content; accepts `&str`, `String`, or
    ///   [`crate::event::MessageChain`]; see [`crate::event::SendMessage::chain`] for the
    ///   conversion rules.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable,
    /// or [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors are returned when waiting for the response.
    pub async fn send_private_message(
        &self,
        user_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        let msg = json!(
            {
                "user_id": user_id,
                "message": message.chain(),
            }
        );
        self.websocket_send("send_private_msg", msg)
            .await
            .map(|r| SendMessageAsyncResponse(r))
    }

    /// Sends a group message and returns a handle for waiting on the send result.
    ///
    /// A usable connection must already be established before calling this method. Returning
    /// `Ok` only means the request was written to the current WebSocket; it does not mean the
    /// message was delivered. Call [`SendMessageAsyncResponse::wait_response`] when the
    /// message ID is needed.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
    /// - `message`: message content; accepts `&str`, `String`, or
    ///   [`crate::event::MessageChain`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable,
    /// or [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    pub async fn send_group_message(
        &self,
        group_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
                "message": message.chain(),
            }
        );
        self.websocket_send("send_group_msg", msg)
            .await
            .map(|r| SendMessageAsyncResponse(r))
    }

    /// Selects the private or group send method based on the message type.
    ///
    /// [`MessageType::Private`] calls [`Self::send_private_message`] with `target_id` as the
    /// QQ ID; [`MessageType::Group`] calls [`Self::send_group_message`] with it as the group
    /// ID. Other values (including [`MessageType::Unknown`]) return a field error locally and
    /// do not send a request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::FieldError`] when the message type is unsupported;
    /// other errors match those of the selected send method.
    pub async fn send_message(
        &self,
        message_type: MessageType,
        target_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        match message_type {
            MessageType::Private => self.send_private_message(target_id, message).await,
            MessageType::Group => self.send_group_message(target_id, message).await,
            _ => Err(Error::FieldError("unknown message_type".to_string())),
        }
    }
}
