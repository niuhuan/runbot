//! OneBot 11 event, response, and message segment types.
//!
//! [`Post`] is the main entry point for content received over the WebSocket; it can be parsed from
//! JSON text with [`parse_post`]. [`Message`] represents a chat message, and [`MessageData`]
//! represents a message segment within it; [`SendMessage`] and [`MessageChain`] are used to build
//! outgoing content. Notices and requests are represented by [`Notice`] and [`Request`]
//! respectively.
//!
//! Event parsing is defined by each type's `parse` method; some types also derive `Deserialize`,
//! but the derived format is not necessarily the OneBot wire format.

use crate::error::{Error, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

/// A single OneBot report or API response received over the WebSocket.
///
/// It is usually constructed from a JSON value via [`parse_post`] or [`Post::parse`]. Dispatch
/// checks the `retcode` field first: when present, the value is parsed as a [`Response`]; otherwise
/// a string `post_type` is required, and dispatch follows `meta_event`, `message`, `notice`,
/// `request`, and `message_sent`. An unknown `post_type` is kept as [`Post::Unknown`]; field errors
/// inside a known type are not downgraded to Unknown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Post {
    /// Meta event, including heartbeat and lifecycle events.
    MetaEvent(MetaEvent),
    /// API response, identified by the presence of the `retcode` field.
    Response(Response),
    /// Received chat message event.
    Message(Message),
    /// Notice event, such as a group member change, recall, or poke.
    Notice(Notice),
    /// Request event, such as a friend request or group join request.
    Request(Request),
    /// The `message_sent` report sent after the bot itself sends a message; the payload is the same
    /// as [`Message`].
    MessageSent(Message),
    /// Unrecognized `post_type`; stores the complete raw JSON object.
    Unknown(serde_json::Value),
}

/// The `post_type` tag of a OneBot event.
///
/// During deserialization, known tags map to the corresponding variant, and unknown strings are
/// stored in [`PostType::Unknown`] and serialized back unchanged. This type only describes the tag
/// and does not carry event data.
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum PostType {
    /// `meta_event`: heartbeat or lifecycle event.
    MetaEvent,
    /// `response`: API response tag; actual response dispatch is determined by the `retcode` field.
    Response,
    /// `message`: chat message event.
    Message,
    /// `message_sent`: report sent after the bot itself sends a message.
    MessageSent,
    /// `notice`: notice event.
    Notice,
    /// `request`: request event.
    Request,
    /// Unrecognized tag; stores the raw string.
    Unknown(String),
}

impl Default for PostType {
    fn default() -> Self {
        PostType::Unknown("".to_string())
    }
}

/// OneBot meta event, distinguished by `meta_event_type` as a heartbeat or lifecycle event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaEvent {
    /// Lifecycle event, such as connection enable, disable, or reconnect.
    Lifecycle(Lifecycle),
    /// Periodic heartbeat event carrying the bot's online status.
    Heartbeat(Heartbeat),
}

/// Periodic heartbeat event.
///
/// Parsed from a OneBot report via [`Heartbeat::parse`]; `status` is a nested object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Time the event occurred; the OneBot convention is a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::MetaEvent`] (written during parsing).
    pub post_type: PostType,
    /// Online status of the bot.
    pub status: HeartbeatStatus,
    /// Heartbeat interval in milliseconds, determined by the upstream configuration.
    pub interval: i64,
}

/// Online status within a heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatStatus {
    /// Whether the account is online.
    pub online: bool,
    /// Whether the account status is good; the exact determination is up to the upstream
    /// implementation.
    pub good: bool,
}

/// Lifecycle event, such as a OneBot implementation being enabled, disabled, or reconnected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    /// Time the event occurred; the OneBot convention is a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::MetaEvent`] (written during parsing).
    pub post_type: PostType,
    /// Lifecycle subtype.
    pub sub_type: LifecycleSubType,
}

/// Lifecycle subtype.
///
/// [`Lifecycle::parse`] accepts the lowercase `enable`, `disable`, and `connect` values used by
/// OneBot; the derived deserializer matches variant names and should not be used directly to parse
/// the wire protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleSubType {
    /// The OneBot implementation has been enabled.
    Enable,
    /// The OneBot implementation has been disabled.
    Disable,
    /// The OneBot implementation has established a connection.
    Connect,
}

/// OneBot API response.
///
/// [`Response::parse`] requires the `status`, `retcode`, `message`, `wording`, and `echo` fields; a
/// missing `data` field is stored as [`serde_json::Value::Null`]. The OneBot convention is that
/// `retcode == 0` means success, but this type does not check that value itself; the check happens
/// in [`crate::bot_context::EchoAsyncResponse::data`] and
/// [`crate::bot_context::EchoAsyncResponse::ok`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Upstream status string, usually `ok` or `failed`; this library does not use it to determine
    /// success.
    pub status: String,
    /// Business return code; `0` usually means success.
    pub retcode: i64,
    /// Response data; `null` when missing, with the concrete structure determined by the
    /// corresponding API.
    pub data: serde_json::Value,
    /// Error or status description, usually an empty string.
    pub message: String,
    /// User-facing error description, possibly empty.
    pub wording: String,
    /// Echo identifier corresponding to the request, used to match a pending response.
    pub echo: String,
}

/// Chat message event.
///
/// Parsed from a OneBot `message` or `message_sent` report via [`Message::parse`]. This struct
/// derives both `Serialize` and `Deserialize`, but the derived deserialization format of
/// [`MessageData`] differs from the OneBot wire protocol, so events should use `parse` rather than
/// `serde_json::from_value` directly.
///
/// The order of message segments is preserved in [`Message::message`]; the library does not merge
/// or reorder them automatically.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Sender's QQ number; used to identify the sender in both private and group chats.
    pub user_id: i64,
    /// Time the event occurred; the OneBot convention is a Unix timestamp in seconds.
    pub time: i64,
    /// Message ID, usable for recall, reply, or fetching message details.
    pub message_id: i64,
    /// Message sequence number, used by some upstreams for history pagination; not necessarily
    /// globally unique.
    pub message_seq: i64,
    /// Message type: private or group.
    pub message_type: MessageType,
    /// Sender profile.
    pub sender: Sender,
    /// Raw text representation of the message; the exact format is determined by the upstream and
    /// is not guaranteed to equal the concatenation of message segments.
    pub raw_message: String,
    /// Font identifier provided by the upstream; this library does not interpret its meaning.
    pub font: i64,
    /// Message subtype, such as a friend private message or a normal group message.
    pub sub_type: MessageSubType,
    /// List of message segments, kept in their original order.
    pub message: Vec<MessageData>,
    /// Message segment format, usually `array`; the upstream may report `string`.
    pub message_format: MessageFormat,
    /// Event tag, usually [`PostType::Message`] or [`PostType::MessageSent`].
    pub post_type: PostType,
    /// Group number; [`Message::parse`] writes `0` when the field is missing or the message is
    /// private.
    pub group_id: i64,
}

/// Message type tag.
///
/// [`Message::parse`] accepts only `private` and `group`; any other value returns a field error.
/// During direct deserialization, unknown strings are kept as [`MessageType::Unknown`].
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum MessageType {
    /// Private message.
    Private,
    /// Group message.
    Group,
    /// Unrecognized message type; stores the raw string.
    Unknown(String),
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Unknown("".to_string())
    }
}

/// Message segment format tag.
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum MessageFormat {
    /// Message segment array format.
    Array,
    /// String format; the exact encoding is determined by the upstream.
    String,
    /// Unrecognized format; stores the raw string.
    Unknown(String),
}

/// Message subtype tag.
///
/// [`Message::parse`] accepts only `friend` and `normal`; an unknown subtype causes a field error.
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum MessageSubType {
    /// Friend private message.
    Friend,
    /// Normal group message.
    Normal,
    /// Unrecognized subtype; stores the raw string.
    Unknown(String),
}

impl Default for MessageSubType {
    fn default() -> Self {
        MessageSubType::Unknown("".to_string())
    }
}

impl Default for MessageFormat {
    fn default() -> Self {
        MessageFormat::Unknown("".to_string())
    }
}

/// Message sender profile.
///
/// Parsed via [`Sender::parse`]; `user_id` is required, and missing remaining fields use empty
/// strings.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson, Default)]
pub struct Sender {
    /// Sender's QQ number.
    pub user_id: i64,
    /// Sender nickname; an empty string when missing.
    #[serde(default)]
    pub nickname: String,
    /// Group card name; an empty string in private chats or when unset.
    #[serde(default)]
    pub card: String,
    /// Group role string, such as `owner`, `admin`, or `member`; usually empty in private chats.
    #[serde(default)]
    pub role: String,
    /// Group title; an empty string when absent.
    #[serde(default)]
    pub title: String,
}

/// OneBot message segment.
///
/// The wire format is `{"type": "...", "data": {...}}`. [`MessageData::parse`] dispatches by
/// `type`; for an unknown type, the contents of `data` (not the complete message segment) are
/// stored as [`MessageData::Unknown`].
///
/// The `Serialize` implementation emits the wire format above. The derived `Deserialize` uses
/// Rust's externally tagged enum representation (for example, `{"Text": {"text": "..."}}`), which
/// differs from the OneBot wire protocol; use [`MessageData::parse`] or [`Message::parse`] to parse
/// events.
#[derive(Debug, Clone, Deserialize)]
pub enum MessageData {
    /// Text segment; `type` is `text`.
    Text(MessageText),
    /// QQ face segment; `type` is `face`.
    Face(MessageFace),
    /// Image segment; `type` is `image`.
    Image(MessageImage),
    /// Voice segment; `type` is `record`.
    Record(MessageRecord),
    /// Short video segment; `type` is `video`.
    Video(MessageVideo),
    /// Mention segment; `type` is `at`.
    At(MessageAt),
    /// Reply reference segment; `type` is `reply`.
    Reply(MessageReply),
    /// Merged forward segment; `type` is `forward`.
    Forward(MessageForward),
    /// JSON card segment; `type` is `json`.
    Json(MessageJson),
    /// XML card segment; `type` is `xml`.
    Xml(MessageXml),
    /// Unrecognized segment; stores the raw `data` object. Serialization emits only that object,
    /// without the original `type`.
    Unknown(serde_json::Value),
}

impl serde::Serialize for MessageData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MessageData::Text(text) => json!({
                "type": "text",
                "data": text,
            })
            .serialize(serializer),
            MessageData::Face(face) => json!({
                "type": "face",
                "data": face,
            })
            .serialize(serializer),
            MessageData::Image(image) => json!({
                "type": "image",
                "data": image,
            })
            .serialize(serializer),
            MessageData::Record(record) => json!({
                "type": "record",
                "data": record,
            })
            .serialize(serializer),
            MessageData::Video(video) => json!({
                "type": "video",
                "data": video,
            })
            .serialize(serializer),
            MessageData::At(at) => json!({
                "type": "at",
                "data": at,
            })
            .serialize(serializer),
            MessageData::Reply(reply) => json!({
                "type": "reply",
                "data": reply,
            })
            .serialize(serializer),
            MessageData::Forward(forward) => json!({
                "type": "forward",
                "data": forward,
            })
            .serialize(serializer),
            MessageData::Json(json) => json!({
                "type": "json",
                "data": json,
            })
            .serialize(serializer),
            MessageData::Xml(xml) => json!({
                "type": "xml",
                "data": xml,
            })
            .serialize(serializer),
            MessageData::Unknown(value) => value.clone().serialize(serializer),
        }
    }
}

/// Text message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageText {
    /// Text content, with no escaping or length limit.
    pub text: String,
}

impl MessageText {
    /// Creates a text message segment.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Into<MessageData> for MessageText {
    fn into(self) -> MessageData {
        MessageData::Text(self)
    }
}

/// QQ face message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageFace {
    /// Face ID, per the upstream face table.
    pub id: String,
    /// Face subtype; `0` when missing.
    #[serde(default)]
    pub sub_type: i64,
    /// Raw information attached by the upstream; `null` when missing.
    #[serde(default)]
    pub raw: serde_json::Value,
}

impl Into<MessageData> for MessageFace {
    fn into(self) -> MessageData {
        MessageData::Face(self)
    }
}

/// Image message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageImage {
    /// Image file identifier, which may be a path, URL, or upstream-internal ID.
    pub file: String,
    /// Image subtype; `0` when missing, with the meaning determined by the upstream.
    #[serde(default)]
    pub sub_type: i64,
    /// Image URL; an empty string when missing.
    #[serde(default)]
    pub url: String,
    /// File size, in units determined by the upstream; accepts a number or a numeric string; `0`
    /// when missing.
    #[serde(default, deserialize_with = "crate::common::fuzzy_int")]
    pub file_size: i64,
}

/// Voice message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageRecord {
    /// Voice file identifier.
    pub file: String,
    /// Local path (if provided by the upstream); an empty string when missing.
    #[serde(default)]
    pub path: String,
    /// Voice URL; an empty string when missing.
    #[serde(default)]
    pub url: String,
    /// File size, in units determined by the upstream; `0` when missing.
    #[serde(default, deserialize_with = "crate::common::fuzzy_int")]
    pub file_size: i64,
}

/// Short video message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageVideo {
    /// Video file identifier.
    pub file: String,
    /// Local path (if provided by the upstream); an empty string when missing.
    #[serde(default)]
    pub path: String,
    /// Video URL; an empty string when missing.
    #[serde(default)]
    pub url: String,
    /// File size, in units determined by the upstream; `0` when missing.
    #[serde(default, deserialize_with = "crate::common::fuzzy_int")]
    pub file_size: i64,
}

impl MessageImage {
    /// Creates an image message segment from a file identifier; `sub_type`, `url`, and `file_size`
    /// use zero values.
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            sub_type: 0,
            url: "".into(),
            file_size: 0,
        }
    }
}

impl Into<MessageData> for MessageImage {
    fn into(self) -> MessageData {
        MessageData::Image(self)
    }
}

/// Mention message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageAt {
    /// QQ number of the mentioned user; `all` usually refers to all members, as determined by the
    /// upstream.
    pub qq: String,
    /// Display name of the mentioned user; an empty string when missing.
    #[serde(default)]
    pub name: String,
}

/// Reply reference message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageReply {
    /// ID of the message being replied to; accepts a number or a numeric string.
    #[serde(deserialize_with = "crate::common::fuzzy_int")]
    pub id: i64,
}

impl Into<MessageData> for MessageReply {
    fn into(self) -> MessageData {
        MessageData::Reply(self)
    }
}

/// Merged forward message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageForward {
    /// Merged forward ID, usable with [`crate::onebot11_api::get_forward_msg`] to fetch the
    /// contents.
    pub id: String,
}

/// JSON card message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageJson {
    /// JSON text; its structure depends on the card type and is not parsed by this library.
    pub data: String,
}

/// XML card message segment.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageXml {
    /// XML text; its structure depends on the card type and is not parsed by this library.
    pub data: String,
}

/// Complete contents of a merged forward message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardMessage {
    /// Forward content nodes, kept in the order returned by the upstream.
    pub messages: Vec<ForwardMessageNode>,
}

/// A message node within a merged forward message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardMessageNode {
    /// List of message segments in the node.
    pub content: Vec<MessageData>,
    /// Node sender profile.
    pub sender: Sender,
    /// Node message time; the OneBot convention is a Unix timestamp in seconds.
    pub time: i64,
    /// Node message segment format.
    pub message_format: MessageFormat,
    /// Node message type.
    pub message_type: MessageType,
}

/// OneBot notice event.
///
/// Parsed via [`Notice::parse`] by `notice_type`; an unknown notice type stores the complete raw
/// JSON as [`Notice::Unknown`], while field errors inside a known type are returned directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Notice {
    /// Group file upload.
    GroupUpload(GroupUpload),
    /// Group administrator set or unset.
    GroupAdmin(GroupAdmin),
    /// Group member decrease (left voluntarily or kicked).
    GroupDecrease(GroupDecrease),
    /// Group member increase (joined voluntarily or invited).
    GroupIncrease(GroupIncrease),
    /// Group member muted or unmuted.
    GroupBan(GroupBan),
    /// Friend added.
    FriendAdd(FriendAdd),
    /// Group message recall.
    GroupRecall(GroupRecall),
    /// Friend message recall.
    FriendRecall(FriendRecall),
    /// Notice with subtype `notify`, such as a poke, Lucky King, or group honor.
    Notify(Notify),
    /// Unrecognized `notice_type`; stores the complete raw JSON object.
    Unknown(serde_json::Value),
}

/// OneBot notice type tag.
///
/// Unknown strings are stored in [`NoticeType::Unknown`] and serialized back unchanged.
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde)]
pub enum NoticeType {
    /// `group_upload`: group file upload.
    GroupUpload,
    /// `group_admin`: group administrator change.
    GroupAdmin,
    /// `group_decrease`: group member decrease.
    GroupDecrease,
    /// `group_increase`: group member increase.
    GroupIncrease,
    /// `group_ban`: group member mute change.
    GroupBan,
    /// `friend_add`: friend added.
    FriendAdd,
    /// `group_recall`: group message recall.
    GroupRecall,
    /// `friend_recall`: friend message recall.
    FriendRecall,
    /// `notify`: notice subtype.
    Notify,
    /// Unrecognized notice type; stores the raw string.
    Unknown(String),
}

/// Group file upload notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUpload {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::GroupUpload`].
    pub notice_type: NoticeType,
    /// Group number containing the file.
    pub group_id: i64,
    /// Uploader's QQ number.
    pub user_id: i64,
    /// Information about the uploaded file.
    pub file: GroupUploadFile,
}

/// File information in a group file upload notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUploadFile {
    /// File ID, usable with file-related APIs.
    pub id: String,
    /// File name.
    pub name: String,
    /// File size, in units determined by the upstream.
    pub size: i64,
    /// File business ID, used by the upstream.
    pub busid: i64,
}

/// Group administrator change notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAdmin {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::GroupAdmin`].
    pub notice_type: NoticeType,
    /// Whether an administrator is being set or unset.
    pub sub_type: GroupAdminSubType,
    /// Group number.
    pub group_id: i64,
    /// QQ number of the member whose administrator status is being set or unset.
    pub user_id: i64,
}

/// Group administrator change direction.
///
/// [`GroupAdmin::parse`] accepts `set` and `unset`; the derived deserializer matches variant names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupAdminSubType {
    /// Set an administrator.
    Set,
    /// Unset an administrator.
    UnSet,
}

/// Group member decrease notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDecrease {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::GroupDecrease`].
    pub notice_type: NoticeType,
    /// Reason for the member decrease.
    pub sub_type: GroupDecreaseSubType,
    /// Group number.
    pub group_id: i64,
    /// Operator's QQ number; usually equal to `user_id` when a member leaves voluntarily.
    pub operator_id: i64,
    /// QQ number of the member who left the group.
    pub user_id: i64,
}

/// Reason for a group member decrease.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupDecreaseSubType {
    /// The member left voluntarily.
    Leave,
    /// The member was kicked.
    Kick,
    /// The bot itself was kicked.
    KickMe,
}

/// Group member increase notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupIncrease {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::GroupIncrease`].
    pub notice_type: NoticeType,
    /// How the member was added.
    pub sub_type: GroupIncreaseSubType,
    /// Group number.
    pub group_id: i64,
    /// Operator's QQ number; usually equal to `user_id` when a member joins voluntarily.
    pub operator_id: i64,
    /// QQ number of the member who joined the group.
    pub user_id: i64,
}

/// How a group member was added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupIncreaseSubType {
    /// An administrator approved the join request.
    Approve,
    /// The member was invited to the group.
    Invite,
}

/// Group member mute change notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupBan {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::GroupBan`].
    pub notice_type: NoticeType,
    /// Mute or unmute.
    pub sub_type: GroupBanSubType,
    /// Group number.
    pub group_id: i64,
    /// Operator's QQ number.
    pub operator_id: i64,
    /// QQ number of the member being muted or unmuted.
    pub user_id: i64,
}

/// Direction of a group member mute change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupBanSubType {
    /// Start a mute.
    Ban,
    /// Lift a mute.
    LiftBan,
}

/// Friend added notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendAdd {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::FriendAdd`].
    pub notice_type: NoticeType,
    /// QQ number of the newly added friend.
    pub user_id: i64,
}

/// Group message recall notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRecall {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::GroupRecall`].
    pub notice_type: NoticeType,
    /// Group number.
    pub group_id: i64,
    /// QQ number of the original message sender.
    pub user_id: i64,
    /// QQ number of the operator who performed the recall.
    pub operator_id: i64,
    /// ID of the recalled message.
    pub message_id: i64,
}

/// Friend message recall notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRecall {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::FriendRecall`].
    pub notice_type: NoticeType,
    /// QQ number of the friend who recalled the message.
    pub user_id: i64,
    /// ID of the recalled message.
    pub message_id: i64,
}

/// Sub-notice when `notice_type` is `notify`.
///
/// Parsed via [`Notify::parse`] by `sub_type`; an unknown subtype stores the complete raw JSON. A
/// known subtype that is missing required fields returns a field error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Notify {
    /// Poke notice.
    Poke(Poke),
    /// Lucky King notice.
    LuckyKing(LuckyKing),
    /// Group honor notice.
    Honor(Honor),
    /// Unrecognized notice subtype; stores the complete raw JSON object.
    Unknown(serde_json::Value),
}

/// Subtype tag of a `notify` notice.
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde)]
pub enum NotifySubType {
    /// `poke`: poke.
    Poke,
    /// `lucky_king`: Lucky King.
    LuckyKing,
    /// `honor`: group honor.
    Honor,
    /// Unrecognized subtype; stores the raw string.
    Unknown(String),
}

/// Poke notice.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct Poke {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::Notify`].
    pub notice_type: NoticeType,
    /// Always [`NotifySubType::Poke`].
    pub sub_type: NotifySubType,
    /// Group number; `0` when the field is missing for a private chat poke.
    #[serde(default)]
    pub group_id: i64,
    /// QQ number of the user who initiated the poke.
    pub user_id: i64,
    /// QQ number of the user being poked.
    pub target_id: i64,
}

/// Lucky King notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuckyKing {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::Notify`].
    pub notice_type: NoticeType,
    /// Always [`NotifySubType::LuckyKing`].
    pub sub_type: NotifySubType,
    /// Group number.
    pub group_id: i64,
    /// QQ number of the red packet sender.
    pub user_id: i64,
    /// QQ number of the Lucky King.
    pub target_id: i64,
}

/// Group honor notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Honor {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Notice`].
    pub post_type: PostType,
    /// Always [`NoticeType::Notify`].
    pub notice_type: NoticeType,
    /// Always [`NotifySubType::Honor`].
    pub sub_type: NotifySubType,
    /// Group number.
    pub group_id: i64,
    /// Honor type.
    pub honor_type: HonorType,
    /// QQ number of the member who received the honor.
    pub user_id: i64,
}

/// Group honor type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HonorType {
    /// Dragon King (active in group chat).
    Talkative,
    /// Group Chat Fire (consistently active in chat).
    Performer,
    /// Group Chat Blazing Flame (an emotional honor).
    Emotion,
}

/// OneBot request event.
///
/// The `parse` generated by `#[enum_field(name = "request_type")]` dispatches by `request_type`;
/// `friend` and `group` are parsed into the corresponding structs, and an unknown value stores the
/// complete raw JSON as [`Request::Unknown`].
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::UnknownEnumSerdeAndParse)]
#[enum_field(name = "request_type")]
pub enum Request {
    /// Friend request; `request_type` is `friend`.
    Friend(FriendRequest),
    /// Group join or invite request; `request_type` is `group`.
    Group(GroupRequest),
    /// Unrecognized request type; stores the complete raw JSON object.
    Unknown(serde_json::Value),
}

/// Friend request.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct FriendRequest {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Request`].
    pub post_type: PostType,
    /// Always [`RequestType::Friend`].
    pub request_type: RequestType,
    /// QQ number of the request sender.
    pub user_id: i64,
    /// Verification message.
    pub comment: String,
    /// Request identifier, used with [`crate::onebot11_api::set_friend_add_request`].
    pub flag: String,
}

/// Group join or invite request.
#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct GroupRequest {
    /// Event time, a Unix timestamp in seconds.
    pub time: i64,
    /// QQ number of the bot that received the event.
    pub self_id: i64,
    /// Always [`PostType::Request`].
    pub post_type: PostType,
    /// Always [`RequestType::Group`].
    pub request_type: RequestType,
    /// Request direction: a request to join the group or an invitation to the group.
    pub sub_type: GroupRequestSubType,
    /// Group number.
    pub group_id: i64,
    /// QQ number of the applicant or invitee.
    pub user_id: i64,
    /// Verification message.
    pub comment: String,
    /// Request identifier, used with [`crate::onebot11_api::set_group_add_request`].
    pub flag: String,
}

/// Request type tag.
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum RequestType {
    /// `friend`: friend request.
    Friend,
    /// `group`: group join or invite request.
    Group,
    /// Unrecognized request type; stores the raw string.
    Unknown(String),
}

/// Group request direction.
#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum GroupRequestSubType {
    /// `add`: a user is requesting to join the group.
    Add,
    /// `invite`: a user is being invited to the group.
    Invite,
    /// Unrecognized direction; stores the raw string.
    Unknown(String),
}

impl Post {
    /// Parses a JSON value into a concrete report type by `retcode` or `post_type`.
    ///
    /// If the `retcode` field is present, the value is handled by [`Response::parse`]; otherwise a
    /// string `post_type` is required. An unknown `post_type` returns [`Post::Unknown`] and keeps
    /// the complete object. Known types are parsed by their corresponding handwritten `parse`
    /// methods, and field errors inside them return [`Error::FieldError`] rather than being
    /// downgraded to Unknown.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when `post_type` is missing or is not a string; when a known
    /// event or response fails to parse, returns the error from the corresponding parse method.
    pub fn parse(value: &serde_json::Value) -> Result<Post> {
        if value.get("retcode").is_some() {
            return Ok(Post::Response(Response::parse(value)?));
        }
        let post_type = value.get("post_type").ok_or(Error::FieldError(format!(
            "post_type not found: {:?}",
            value
        )))?;
        let post_type = post_type.as_str().ok_or(Error::FieldError(format!(
            "post_type not found: {:?}",
            value
        )))?;
        match post_type {
            "meta_event" => Ok(Post::MetaEvent(MetaEvent::parse(&value)?)),
            "message" => Ok(Post::Message(Message::parse(&value)?)),
            "notice" => Ok(Post::Notice(Notice::parse(&value)?)),
            "request" => Ok(Post::Request(Request::parse(&value)?)),
            "message_sent" => Ok(Post::MessageSent(Message::parse(&value)?)),
            _ => Ok(Post::Unknown(value.clone())),
        }
    }
}

impl MetaEvent {
    /// Parses a heartbeat or lifecycle event by `meta_event_type`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when `meta_event_type` is missing or is not a string; an
    /// unknown value likewise returns a field error instead of being downgraded to Unknown.
    pub fn parse(value: &serde_json::Value) -> Result<MetaEvent> {
        let meta_event_type = value
            .get("meta_event_type")
            .ok_or(Error::FieldError("meta_event_type not found".to_string()))?;
        let meta_event_type = meta_event_type
            .as_str()
            .ok_or(Error::FieldError("meta_event_type not found".to_string()))?;
        match meta_event_type {
            "heartbeat" => Ok(MetaEvent::Heartbeat(Heartbeat::parse(value)?)),
            "lifecycle" => Ok(MetaEvent::Lifecycle(Lifecycle::parse(value)?)),
            _ => Err(Error::FieldError(format!(
                "unknown meta_event_type: {}",
                meta_event_type
            ))),
        }
    }
}

impl Lifecycle {
    /// Parses a lifecycle event.
    ///
    /// Requires `time` and `self_id` as integers and `sub_type` as `enable`, `disable`, or
    /// `connect`; `post_type` is written as [`PostType::MetaEvent`] in the parse result.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or `sub_type` is
    /// unknown.
    pub fn parse(value: &serde_json::Value) -> Result<Lifecycle> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let sub_type = value
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_str()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = match sub_type {
            "enable" => LifecycleSubType::Enable,
            "disable" => LifecycleSubType::Disable,
            "connect" => LifecycleSubType::Connect,
            _ => return Err(Error::FieldError(format!("unknown sub_type: {}", sub_type))),
        };
        Ok(Lifecycle {
            time,
            self_id,
            post_type: PostType::MetaEvent,
            sub_type,
        })
    }
}

impl Heartbeat {
    /// Parses a heartbeat event.
    ///
    /// Requires the `time`, `self_id`, `status`, and `interval` fields; `status` must contain the
    /// boolean fields `online` and `good`. `post_type` is written as [`PostType::MetaEvent`] in the
    /// parse result.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing or has the wrong type.
    pub fn parse(value: &serde_json::Value) -> Result<Heartbeat> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let status = value
            .get("status")
            .ok_or(Error::FieldError("status not found".to_string()))?;
        let status = HeartbeatStatus::parse(status)?;
        let interval = value
            .get("interval")
            .ok_or(Error::FieldError("interval not found".to_string()))?;
        let interval = interval
            .as_i64()
            .ok_or(Error::FieldError("interval not found".to_string()))?;
        Ok(Heartbeat {
            time,
            self_id,
            post_type: PostType::MetaEvent,
            status,
            interval,
        })
    }
}

impl HeartbeatStatus {
    /// Parses a heartbeat status object.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when `online` or `good` is missing or is not a boolean.
    pub fn parse(value: &serde_json::Value) -> Result<HeartbeatStatus> {
        let online = value
            .get("online")
            .ok_or(Error::FieldError("online not found".to_string()))?;
        let online = online
            .as_bool()
            .ok_or(Error::FieldError("online not found".to_string()))?;
        let good = value
            .get("good")
            .ok_or(Error::FieldError("good not found".to_string()))?;
        let good = good
            .as_bool()
            .ok_or(Error::FieldError("good not found".to_string()))?;
        Ok(HeartbeatStatus { online, good })
    }
}

impl Response {
    /// Parses an API response object.
    ///
    /// A missing `data` field uses [`serde_json::Value::Null`]; all other fields are required.
    ///
    /// # Errors
    ///
    /// When `status`, `retcode`, `message`, `wording`, or `echo` is missing or has the wrong type,
    /// this method returns [`Error::FieldError`].
    pub fn parse(value: &serde_json::Value) -> Result<Response> {
        let status = value
            .get("status")
            .ok_or(Error::FieldError("status not found".to_string()))?;
        let status = status
            .as_str()
            .ok_or(Error::FieldError("status not found".to_string()))?;
        let retcode = value
            .get("retcode")
            .ok_or(Error::FieldError("retcode not found".to_string()))?;
        let retcode = retcode
            .as_i64()
            .ok_or(Error::FieldError("retcode not found".to_string()))?;
        let data = if let Some(data) = value.get("data") {
            data.clone()
        } else {
            serde_json::Value::Null
        };
        let message = value
            .get("message")
            .ok_or(Error::FieldError("message not found".to_string()))?;
        let message = message
            .as_str()
            .ok_or(Error::FieldError("message not found".to_string()))?;
        let wording = value
            .get("wording")
            .ok_or(Error::FieldError("wording not found".to_string()))?;
        let wording = wording
            .as_str()
            .ok_or(Error::FieldError("wording not found".to_string()))?;
        let echo = value
            .get("echo")
            .ok_or(Error::FieldError("echo not found".to_string()))?;
        let echo = echo
            .as_str()
            .ok_or(Error::FieldError("echo not found".to_string()))?;
        Ok(Response {
            status: status.to_string(),
            retcode,
            data,
            message: message.to_string(),
            wording: wording.to_string(),
            echo: echo.to_string(),
        })
    }
}

impl Message {
    /// Parses a OneBot `message` or `message_sent` event.
    ///
    /// Requires `self_id`, `user_id`, `time`, `message_id`, `message_seq`, `message_type`,
    /// `sender`, `raw_message`, `font`, `sub_type`, `message`, `message_format`, and `post_type`.
    /// `message_type` accepts only `private`/`group`, and `sub_type` accepts only
    /// `friend`/`normal`; `group_id` may be missing, in which case it is written as `0`. Message
    /// segments are parsed one by one via [`MessageData::parse`].
    ///
    /// # Errors
    ///
    /// When a required field is missing or has the wrong type, the message type or subtype is
    /// unknown, or any message segment fails to parse, this method returns [`Error::FieldError`].
    pub fn parse(value: &serde_json::Value) -> Result<Message> {
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let message_id = value
            .get("message_id")
            .ok_or(Error::FieldError("message_id not found".to_string()))?;
        let message_id = message_id
            .as_i64()
            .ok_or(Error::FieldError("message_id not found".to_string()))?;
        let message_seq = value
            .get("message_seq")
            .ok_or(Error::FieldError("message_seq not found".to_string()))?;
        let message_seq = message_seq
            .as_i64()
            .ok_or(Error::FieldError("message_seq not found".to_string()))?;
        let message_type = value
            .get("message_type")
            .ok_or(Error::FieldError("message_type not found".to_string()))?;
        let message_type = message_type
            .as_str()
            .ok_or(Error::FieldError("message_type not found".to_string()))?;
        let message_type = match message_type {
            "private" => MessageType::Private,
            "group" => MessageType::Group,
            _ => {
                return Err(Error::FieldError(format!(
                    "unknown message_type: {}",
                    message_type
                )));
            }
        };
        let sender = value
            .get("sender")
            .ok_or(Error::FieldError("sender not found".to_string()))?;
        let sender = Sender::parse(sender)?;
        let raw_message = value
            .get("raw_message")
            .ok_or(Error::FieldError("raw_message not found".to_string()))?;
        let raw_message = raw_message
            .as_str()
            .ok_or(Error::FieldError("raw_message not found".to_string()))?;
        let font = value
            .get("font")
            .ok_or(Error::FieldError("font not found".to_string()))?;
        let font = font
            .as_i64()
            .ok_or(Error::FieldError("font not found".to_string()))?;
        let sub_type = value
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_str()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = match sub_type {
            "friend" => MessageSubType::Friend,
            "normal" => MessageSubType::Normal,
            _ => return Err(Error::FieldError(format!("unknown sub_type: {}", sub_type))),
        };
        let message = value
            .get("message")
            .ok_or(Error::FieldError("message not found".to_string()))?;
        let message = message
            .as_array()
            .ok_or(Error::FieldError("message not found".to_string()))?;
        let message = message
            .iter()
            .map(|m| MessageData::parse(m))
            .collect::<Result<Vec<MessageData>>>()?;
        let message_format = value
            .get("message_format")
            .ok_or(Error::FieldError("message_format not found".to_string()))?;
        let message_format = MessageFormat::parse(message_format)?;
        let post_type = value
            .get("post_type")
            .ok_or(Error::FieldError("post_type not found".to_string()))?;
        let post_type = PostType::parse(post_type)?;
        let group_id = if let Some(group_id) = value.get("group_id") {
            group_id
                .as_i64()
                .ok_or(Error::FieldError("group_id not found".to_string()))?
        } else {
            0
        };
        Ok(Message {
            self_id,
            user_id,
            time,
            message_id,
            message_seq,
            message_type,
            sender,
            raw_message: raw_message.to_string(),
            font,
            sub_type,
            message,
            message_format,
            post_type: post_type,
            group_id,
        })
    }
}

impl MessageData {
    /// Parses a single OneBot message segment.
    ///
    /// Requires the object to contain a string `type` and a `data` field, which may be an object or
    /// any other value. A known `type` is passed to the `parse` method of the corresponding struct;
    /// an unknown `type` returns [`MessageData::Unknown`] containing the contents of `data`, not
    /// the complete message segment.
    ///
    /// # Errors
    ///
    /// When `type` or `data` is missing, `type` is not a string, or `data` for a known type fails
    /// to parse, this method returns [`Error::FieldError`].
    pub fn parse(value: &serde_json::Value) -> Result<MessageData> {
        let r#type = value
            .get("type")
            .ok_or(Error::FieldError("type not found".to_string()))?;
        let r#type = r#type
            .as_str()
            .ok_or(Error::FieldError("type not found".to_string()))?;
        let value = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        match r#type {
            "text" => Ok(MessageData::Text(MessageText::parse(value)?)),
            "face" => Ok(MessageData::Face(MessageFace::parse(value)?)),
            "image" => Ok(MessageData::Image(MessageImage::parse(value)?)),
            "record" => Ok(MessageData::Record(MessageRecord::parse(value)?)),
            "video" => Ok(MessageData::Video(MessageVideo::parse(value)?)),
            "at" => Ok(MessageData::At(MessageAt::parse(value)?)),
            "reply" => Ok(MessageData::Reply(MessageReply::parse(value)?)),
            "forward" => Ok(MessageData::Forward(MessageForward::parse(value)?)),
            "json" => Ok(MessageData::Json(MessageJson::parse(value)?)),
            "xml" => Ok(MessageData::Xml(MessageXml::parse(value)?)),
            _ => Ok(MessageData::Unknown(value.clone())),
        }
    }
}

/// Parses JSON text into an event or API response.
///
/// JSON syntax is parsed first, then dispatch follows the rules of [`Post::parse`]. If the
/// `retcode` field is present, the value is parsed as a response; other objects require a string
/// `post_type`. An unknown `post_type` is kept as [`Post::Unknown`], and data errors in known types
/// are not uniformly downgraded to Unknown.
///
/// # Errors
///
/// JSON syntax or deserialization errors return [`Error::JsonError`]; missing dispatch fields, type
/// errors, and field errors in handwritten parsing return [`Error::FieldError`].
///
/// # Examples
///
/// ```rust
/// use runbot::event::{parse_post, Post};
///
/// # fn main() -> runbot::error::Result<()> {
/// let post = parse_post(r#"{"post_type":"vendor_event","value":1}"#)?;
/// assert!(matches!(post, Post::Unknown(_)));
/// # Ok(())
/// # }
/// ```
pub fn parse_post(text: &str) -> Result<Post> {
    let value: serde_json::Value = serde_json::from_str(text)?;
    Post::parse(&value)
}

/// Input that can be converted into a OneBot message chain.
///
/// [`chain`](SendMessage::chain) consumes the input and returns a [`MessageChain`]. `&str`,
/// `String`, and `&String` are converted into a single text segment; an existing [`MessageChain`]
/// is returned unchanged. This trait only performs type conversion; it does not send messages or
/// validate message content or length.
pub trait SendMessage: Sync + Send {
    /// Converts `self` into a list of message segments.
    ///
    /// Implementations consume `self`; the returned list preserves the input order.
    fn chain(self) -> MessageChain;
}

impl SendMessage for &str {
    fn chain(self) -> MessageChain {
        vec![MessageData::Text(MessageText {
            text: self.to_string(),
        })]
    }
}

impl SendMessage for String {
    fn chain(self) -> MessageChain {
        vec![MessageData::Text(MessageText { text: self })]
    }
}

impl SendMessage for &String {
    fn chain(self) -> MessageChain {
        vec![MessageData::Text(MessageText { text: self.clone() })]
    }
}

/// An ordered list of message segments to send.
///
/// The message chain is not merged, deduplicated, or escaped; each segment is written to the
/// `message` field of the OneBot request in `Vec` order.
pub type MessageChain = Vec<MessageData>;

impl SendMessage for MessageChain {
    fn chain(self) -> MessageChain {
        self
    }
}

impl Into<MessageData> for &str {
    fn into(self) -> MessageData {
        MessageData::Text(MessageText {
            text: self.to_string(),
        })
    }
}

impl Into<MessageData> for String {
    fn into(self) -> MessageData {
        MessageData::Text(MessageText { text: self })
    }
}

impl ForwardMessage {
    /// Parses the merged forward contents returned by `get_forward_msg`.
    ///
    /// Requires `messages` to be an array, with each element parsed via
    /// [`ForwardMessageNode::parse`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when `messages` is missing or is not an array, or when any
    /// node fails to parse.
    pub fn parse(value: &serde_json::Value) -> Result<ForwardMessage> {
        let messages = value
            .get("messages")
            .ok_or(Error::FieldError("messages not found".to_string()))?;
        let messages = messages
            .as_array()
            .ok_or(Error::FieldError("messages not found".to_string()))?;
        let messages = messages
            .iter()
            .map(|v| ForwardMessageNode::parse(v))
            .collect::<Result<Vec<ForwardMessageNode>>>()?;
        Ok(ForwardMessage { messages })
    }
}

impl ForwardMessageNode {
    /// Parses a node within a merged forward message.
    ///
    /// Requires a `content` array and the `sender`, `time`, `message_format`, and `message_type`
    /// fields.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or a message
    /// segment fails to parse.
    pub fn parse(value: &serde_json::Value) -> Result<ForwardMessageNode> {
        let content = value
            .get("content")
            .ok_or(Error::FieldError("content not found".to_string()))?;
        let content = content
            .as_array()
            .ok_or(Error::FieldError("content not found".to_string()))?;
        let content = content
            .iter()
            .map(|v| MessageData::parse(v))
            .collect::<Result<Vec<MessageData>>>()?;
        let sender = value
            .get("sender")
            .ok_or(Error::FieldError("sender not found".to_string()))?;
        let sender = Sender::parse(sender)?;
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let message_format = value
            .get("message_format")
            .ok_or(Error::FieldError("message_format not found".to_string()))?;
        let message_format = MessageFormat::parse(message_format)?;
        let message_type = value
            .get("message_type")
            .ok_or(Error::FieldError("message_type not found".to_string()))?;
        let message_type = MessageType::parse(message_type)?;
        Ok(ForwardMessageNode {
            content,
            sender,
            time,
            message_format,
            message_type,
        })
    }
}

impl Notice {
    /// Parses a notice event by `notice_type`.
    ///
    /// Known types are passed to the `parse` method of the corresponding struct; an unknown
    /// `notice_type` returns [`Notice::Unknown`] and keeps the complete object.
    ///
    /// # Errors
    ///
    /// When `notice_type` is missing or is not a string, or a field of a known type fails to parse,
    /// this method returns [`Error::FieldError`].
    pub fn parse(value: &serde_json::Value) -> Result<Notice> {
        let notice_type = value
            .get("notice_type")
            .ok_or(Error::FieldError("notice_type not found".to_string()))?;
        let notice_type = notice_type
            .as_str()
            .ok_or(Error::FieldError("notice_type not found".to_string()))?;
        match notice_type {
            "group_upload" => Ok(Notice::GroupUpload(GroupUpload::parse(&value)?)),
            "group_admin" => Ok(Notice::GroupAdmin(GroupAdmin::parse(&value)?)),
            "group_decrease" => Ok(Notice::GroupDecrease(GroupDecrease::parse(&value)?)),
            "group_increase" => Ok(Notice::GroupIncrease(GroupIncrease::parse(&value)?)),
            "group_ban" => Ok(Notice::GroupBan(GroupBan::parse(&value)?)),
            "friend_add" => Ok(Notice::FriendAdd(FriendAdd::parse(&value)?)),
            "group_recall" => Ok(Notice::GroupRecall(GroupRecall::parse(&value)?)),
            "friend_recall" => Ok(Notice::FriendRecall(FriendRecall::parse(&value)?)),
            "notify" => Ok(Notice::Notify(Notify::parse(&value)?)),
            _ => Ok(Notice::Unknown(value.clone())),
        }
    }
}

impl GroupUpload {
    /// Parses a group file upload notice; requires `time`, `self_id`, `group_id`, `user_id`, and
    /// `file`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or the file
    /// information fails to parse.
    pub fn parse(value: &serde_json::Value) -> Result<GroupUpload> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::GroupUpload;
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let file = value
            .get("file")
            .ok_or(Error::FieldError("file not found".to_string()))?;
        let file = GroupUploadFile::parse(&file)?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        Ok(GroupUpload {
            time,
            self_id,
            post_type,
            notice_type,
            group_id,
            user_id,
            file,
        })
    }
}

impl GroupUploadFile {
    /// Parses group file information; requires `id`, `name`, `size`, and `busid`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing or has the wrong type.
    pub fn parse(value: &serde_json::Value) -> Result<GroupUploadFile> {
        let id = value
            .get("id")
            .ok_or(Error::FieldError("id not found".to_string()))?;
        let id = id
            .as_str()
            .ok_or(Error::FieldError("id not found".to_string()))?;
        let name = value
            .get("name")
            .ok_or(Error::FieldError("name not found".to_string()))?;
        let name = name
            .as_str()
            .ok_or(Error::FieldError("name not found".to_string()))?;
        let size = value
            .get("size")
            .ok_or(Error::FieldError("size not found".to_string()))?;
        let size = size
            .as_i64()
            .ok_or(Error::FieldError("size not found".to_string()))?;
        let busid = value
            .get("busid")
            .ok_or(Error::FieldError("busid not found".to_string()))?;
        let busid = busid
            .as_i64()
            .ok_or(Error::FieldError("busid not found".to_string()))?;
        Ok(GroupUploadFile {
            id: id.to_string(),
            name: name.to_string(),
            size,
            busid: busid,
        })
    }
}

impl GroupAdmin {
    /// Parses a group administrator change notice.
    ///
    /// `sub_type` accepts `set` and `unset`; `post_type` and `notice_type` are written by the parse
    /// result.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or `sub_type` is
    /// unknown.
    pub fn parse(value: &serde_json::Value) -> Result<GroupAdmin> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::GroupAdmin;
        let sub_type = value
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_str()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = match sub_type {
            "set" => GroupAdminSubType::Set,
            "unset" => GroupAdminSubType::UnSet,
            _ => return Err(Error::FieldError("sub_type not found".to_string())),
        };
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        Ok(GroupAdmin {
            time,
            self_id,
            post_type,
            notice_type,
            sub_type,
            group_id,
            user_id,
        })
    }
}

impl GroupDecrease {
    /// Parses a group member decrease notice.
    ///
    /// `sub_type` accepts `leave`, `kick`, and `kick_me`; `post_type` and `notice_type` are written
    /// by the parse result accordingly.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or `sub_type` is
    /// unknown.
    pub fn parse(value: &serde_json::Value) -> Result<GroupDecrease> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::GroupDecrease;
        let sub_type = value
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_str()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = match sub_type {
            "leave" => GroupDecreaseSubType::Leave,
            "kick" => GroupDecreaseSubType::Kick,
            "kick_me" => GroupDecreaseSubType::KickMe,
            _ => return Err(Error::FieldError("sub_type not found".to_string())),
        };
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let operator_id = value
            .get("operator_id")
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let operator_id = operator_id
            .as_i64()
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        Ok(GroupDecrease {
            time,
            self_id,
            post_type,
            notice_type,
            sub_type,
            group_id,
            operator_id,
            user_id,
        })
    }
}

impl GroupIncrease {
    /// Parses a group member increase notice.
    ///
    /// `sub_type` accepts `approve` and `invite`; `post_type` and `notice_type` are written by the
    /// parse result.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or `sub_type` is
    /// unknown.
    pub fn parse(value: &serde_json::Value) -> Result<GroupIncrease> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::GroupIncrease;
        let sub_type = value
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_str()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = match sub_type {
            "approve" => GroupIncreaseSubType::Approve,
            "invite" => GroupIncreaseSubType::Invite,
            _ => return Err(Error::FieldError("sub_type not found".to_string())),
        };
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let operator_id = value
            .get("operator_id")
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let operator_id = operator_id
            .as_i64()
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        Ok(GroupIncrease {
            time,
            self_id,
            post_type,
            notice_type,
            sub_type,
            group_id,
            operator_id,
            user_id,
        })
    }
}

impl GroupBan {
    /// Parses a group member mute change notice.
    ///
    /// `sub_type` accepts `ban` and `lift_ban`; `post_type` and `notice_type` are written by the
    /// parse result.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or `sub_type` is
    /// unknown.
    pub fn parse(value: &serde_json::Value) -> Result<GroupBan> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::GroupBan;
        let sub_type = value
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_str()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = match sub_type {
            "ban" => GroupBanSubType::Ban,
            "lift_ban" => GroupBanSubType::LiftBan,
            _ => return Err(Error::FieldError("sub_type not found".to_string())),
        };
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let operator_id = value
            .get("operator_id")
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let operator_id = operator_id
            .as_i64()
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        Ok(GroupBan {
            time,
            self_id,
            post_type,
            notice_type,
            sub_type,
            group_id,
            operator_id,
            user_id,
        })
    }
}

impl FriendAdd {
    /// Parses a friend added notice; requires `time`, `self_id`, and `user_id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing or has the wrong type.
    pub fn parse(value: &serde_json::Value) -> Result<FriendAdd> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::FriendAdd;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        Ok(FriendAdd {
            time,
            self_id,
            post_type,
            notice_type,
            user_id,
        })
    }
}

impl GroupRecall {
    /// Parses a group message recall notice.
    ///
    /// Requires `time`, `self_id`, `group_id`, `user_id`, `operator_id`, and `message_id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing or has the wrong type.
    pub fn parse(value: &serde_json::Value) -> Result<GroupRecall> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::GroupRecall;
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let operator_id = value
            .get("operator_id")
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let operator_id = operator_id
            .as_i64()
            .ok_or(Error::FieldError("operator_id not found".to_string()))?;
        let message_id = value
            .get("message_id")
            .ok_or(Error::FieldError("message_id not found".to_string()))?;
        let message_id = message_id
            .as_i64()
            .ok_or(Error::FieldError("message_id not found".to_string()))?;
        Ok(GroupRecall {
            time,
            self_id,
            post_type,
            notice_type,
            group_id,
            user_id,
            operator_id,
            message_id,
        })
    }
}

impl FriendRecall {
    /// Parses a friend message recall notice.
    ///
    /// Requires `time`, `self_id`, `user_id`, and `message_id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing or has the wrong type.
    pub fn parse(value: &serde_json::Value) -> Result<FriendRecall> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::FriendRecall;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let message_id = value
            .get("message_id")
            .ok_or(Error::FieldError("message_id not found".to_string()))?;
        let message_id = message_id
            .as_i64()
            .ok_or(Error::FieldError("message_id not found".to_string()))?;
        Ok(FriendRecall {
            time,
            self_id,
            post_type,
            notice_type,
            user_id,
            message_id,
        })
    }
}

impl Notify {
    /// Parses a `notify` notice by `sub_type`.
    ///
    /// Known subtypes are passed to the `parse` method of the corresponding struct; an unknown
    /// subtype returns [`Notify::Unknown`] and keeps the complete object.
    ///
    /// # Errors
    ///
    /// When `sub_type` is missing or is not a string, or a field of a known subtype fails to parse,
    /// this method returns [`Error::FieldError`].
    pub fn parse(value: &serde_json::Value) -> Result<Notify> {
        let sub_type = value
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_str()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = match sub_type {
            "poke" => NotifySubType::Poke,
            "lucky_king" => NotifySubType::LuckyKing,
            "honor" => NotifySubType::Honor,
            _ => NotifySubType::Unknown(sub_type.to_string()),
        };
        let notify = match sub_type {
            NotifySubType::Poke => Notify::Poke(Poke::parse(&value)?),
            NotifySubType::LuckyKing => Notify::LuckyKing(LuckyKing::parse(&value)?),
            NotifySubType::Honor => Notify::Honor(Honor::parse(&value)?),
            NotifySubType::Unknown(_) => Notify::Unknown(value.clone()),
        };
        Ok(notify)
    }
}

impl LuckyKing {
    /// Parses a Lucky King notice; requires `time`, `self_id`, `group_id`, `user_id`, and
    /// `target_id`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing or has the wrong type.
    pub fn parse(value: &serde_json::Value) -> Result<LuckyKing> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::Notify;
        let sub_type = NotifySubType::LuckyKing;
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let target_id = value
            .get("target_id")
            .ok_or(Error::FieldError("target_id not found".to_string()))?;
        let target_id = target_id
            .as_i64()
            .ok_or(Error::FieldError("target_id not found".to_string()))?;
        Ok(LuckyKing {
            time,
            self_id,
            post_type,
            notice_type,
            sub_type,
            group_id,
            user_id,
            target_id,
        })
    }
}

impl Honor {
    /// Parses a group honor notice.
    ///
    /// `honor_type` accepts `talkative`, `performer`, and `emotion`; `post_type`, `notice_type`,
    /// and `sub_type` are written by the parse result.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FieldError`] when a field is missing, has the wrong type, or `honor_type`
    /// is unknown.
    pub fn parse(value: &serde_json::Value) -> Result<Honor> {
        let time = value
            .get("time")
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let time = time
            .as_i64()
            .ok_or(Error::FieldError("time not found".to_string()))?;
        let self_id = value
            .get("self_id")
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let self_id = self_id
            .as_i64()
            .ok_or(Error::FieldError("self_id not found".to_string()))?;
        let post_type = PostType::Notice;
        let notice_type = NoticeType::Notify;
        let sub_type = NotifySubType::Honor;
        let group_id = value
            .get("group_id")
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let group_id = group_id
            .as_i64()
            .ok_or(Error::FieldError("group_id not found".to_string()))?;
        let honor_type = value
            .get("honor_type")
            .ok_or(Error::FieldError("honor_type not found".to_string()))?;
        let honor_type = honor_type
            .as_str()
            .ok_or(Error::FieldError("honor_type not found".to_string()))?;
        let honor_type = match honor_type {
            "talkative" => HonorType::Talkative,
            "performer" => HonorType::Performer,
            "emotion" => HonorType::Emotion,
            _ => return Err(Error::FieldError("honor_type not found".to_string())),
        };
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        Ok(Honor {
            time,
            self_id,
            post_type,
            notice_type,
            sub_type,
            group_id,
            honor_type,
            user_id,
        })
    }
}
