use crate::error::{Error, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone)]
pub enum Post {
    MetaEvent(MetaEvent),
    Response(Response),
    Message(Message),
    Notice(Notice),
    Request(Request),
    MessageSent(Message),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum PostType {
    MetaEvent,
    Response,
    Message,
    MessageSent,
    Notice,
    Request,
    Unknown(String),
}

impl Default for PostType {
    fn default() -> Self {
        PostType::Unknown("".to_string())
    }
}

#[derive(Debug, Clone)]
pub enum MetaEvent {
    Lifecycle(Lifecycle),
    Heartbeat(Heartbeat),
}

#[derive(Debug, Clone)]
pub struct Heartbeat {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub status: HeartbeatStatus,
    pub interval: i64,
}

#[derive(Debug, Clone)]
pub struct HeartbeatStatus {
    pub online: bool,
    pub good: bool,
}

#[derive(Debug, Clone)]
pub struct Lifecycle {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub sub_type: LifecycleSubType,
}

#[derive(Debug, Clone)]
pub enum LifecycleSubType {
    Enable,
    Disable,
    Connect,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: String,
    pub retcode: i64,
    pub data: serde_json::Value,
    pub message: String,
    pub wording: String,
    pub echo: String,
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    pub self_id: i64,
    pub user_id: i64,
    pub time: i64,
    pub message_id: i64,
    pub message_seq: i64,
    pub message_type: MessageType,
    pub sender: Sender,
    pub raw_message: String,
    pub font: i64,
    pub sub_type: MessageSubType,
    pub message: Vec<MessageData>,
    pub message_format: MessageFormat,
    pub post_type: PostType,
    pub group_id: i64,
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum MessageType {
    Private,
    Group,
    Unknown(String),
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Unknown("".to_string())
    }
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum MessageFormat {
    Array,
    String,
    Unknown(String),
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum MessageSubType {
    Friend,
    Normal,
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

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson, Default)]
pub struct Sender {
    pub user_id: i64,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub card: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone)]
pub enum MessageData {
    Text(MessageText),
    Face(MessageFace),
    Image(MessageImage),
    Record(MessageRecord),
    Video(MessageVideo),
    At(MessageAt),
    Reply(MessageReply),
    Forward(MessageForward),
    Json(MessageJson),
    Xml(MessageXml),
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

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageText {
    pub text: String,
}

impl MessageText {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Into<MessageData> for MessageText {
    fn into(self) -> MessageData {
        MessageData::Text(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageFace {
    pub id: String,
    pub sub_type: i64,
}

impl Into<MessageData> for MessageFace {
    fn into(self) -> MessageData {
        MessageData::Face(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageImage {
    pub file: String,
    #[serde(default)]
    pub sub_type: i64,
    #[serde(default)]
    pub url: String,
    #[serde(default, deserialize_with = "crate::common::fuzzy_int")]
    pub file_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageRecord {
    pub file: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, deserialize_with = "crate::common::fuzzy_int")]
    pub file_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageVideo {
    pub file: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, deserialize_with = "crate::common::fuzzy_int")]
    pub file_size: i64,
}

impl MessageImage {
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

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageAt {
    pub qq: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageReply {
    pub id: i64,
}

impl Into<MessageData> for MessageReply {
    fn into(self) -> MessageData {
        MessageData::Reply(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageForward {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageJson {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct MessageXml {
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct ForwardMessage {
    pub messages: Vec<ForwardMessageNode>,
}

#[derive(Debug, Clone)]
pub struct ForwardMessageNode {
    pub content: Vec<MessageData>,
    pub sender: Sender,
    pub time: i64,
    pub message_format: MessageFormat,
    pub message_type: MessageType,
}

#[derive(Debug, Clone)]
pub enum Notice {
    GroupUpload(GroupUpload),
    GroupAdmin(GroupAdmin),
    GroupDecrease(GroupDecrease),
    GroupIncrease(GroupIncrease),
    GroupBan(GroupBan),
    FriendAdd(FriendAdd),
    GroupRecall(GroupRecall),
    FriendRecall(FriendRecall),
    Notify(Notify),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde)]
pub enum NoticeType {
    GroupUpload,
    GroupAdmin,
    GroupDecrease,
    GroupIncrease,
    GroupBan,
    FriendAdd,
    GroupRecall,
    FriendRecall,
    Notify,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct GroupUpload {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub group_id: i64,
    pub user_id: i64,
    pub file: GroupUploadFile,
}

#[derive(Debug, Clone)]
pub struct GroupUploadFile {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub busid: i64,
}

#[derive(Debug, Clone)]
pub struct GroupAdmin {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: GroupAdminSubType,
    pub group_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub enum GroupAdminSubType {
    Set,
    UnSet,
}

#[derive(Debug, Clone)]
pub struct GroupDecrease {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: GroupDecreaseSubType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub enum GroupDecreaseSubType {
    Leave,
    Kick,
    KickMe,
}

#[derive(Debug, Clone)]
pub struct GroupIncrease {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: GroupIncreaseSubType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub enum GroupIncreaseSubType {
    Approve,
    Invite,
}

#[derive(Debug, Clone)]
pub struct GroupBan {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: GroupBanSubType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub enum GroupBanSubType {
    Ban,
    LiftBan,
}

#[derive(Debug, Clone)]
pub struct FriendAdd {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub struct GroupRecall {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub group_id: i64,
    pub user_id: i64,
    pub operator_id: i64,
    pub message_id: i64,
}

#[derive(Debug, Clone)]
pub struct FriendRecall {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub user_id: i64,
    pub message_id: i64,
}

#[derive(Debug, Clone)]
pub enum Notify {
    Poke(Poke),
    LuckyKing(LuckyKing),
    Honor(Honor),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde)]
pub enum NotifySubType {
    Poke,
    LuckyKing,
    Honor,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct Poke {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: NotifySubType,
    #[serde(default)]
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}

#[derive(Debug, Clone)]
pub struct LuckyKing {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: NotifySubType,
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}

#[derive(Debug, Clone)]
pub struct Honor {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: NotifySubType,
    pub group_id: i64,
    pub honor_type: HonorType,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub enum HonorType {
    Talkative,
    Performer,
    Emotion,
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::UnknownEnumSerdeAndParse)]
#[enum_field(name = "request_type")]
pub enum Request {
    Friend(FriendRequest),
    Group(GroupRequest),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct FriendRequest {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub request_type: RequestType,
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, runbot_codegen::ParseJson)]
pub struct GroupRequest {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub request_type: RequestType,
    pub sub_type: GroupRequestSubType,
    pub group_id: i64,
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum RequestType {
    Friend,
    Group,
    Unknown(String),
}

#[derive(Debug, Clone, runbot_codegen::UnknownTypeSerde, runbot_codegen::ParseJson)]
pub enum GroupRequestSubType {
    Add,
    Invite,
    Unknown(String),
}

impl Post {
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
            _ => Ok(MessageData::Unknown(value.clone())),
        }
    }
}

pub fn parse_post(text: &str) -> Result<Post> {
    let value: serde_json::Value = serde_json::from_str(text)?;
    Post::parse(&value)
}

pub trait SendMessage {
    fn json(&self) -> Result<serde_json::Value>;
}

impl SendMessage for &str {
    fn json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::Value::String(self.to_string()))
    }
}

impl SendMessage for String {
    fn json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::Value::String(self.clone()))
    }
}

pub type MessageChain = Vec<MessageData>;

impl SendMessage for MessageChain {
    fn json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(&self)?)
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
