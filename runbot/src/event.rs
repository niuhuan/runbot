use crate::error::{Error, Result};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug)]
pub enum Post {
    MetaEvent(MetaEvent),
    Response(Response),
    Message(Message),
    Notice(Notice),
    Request(Request),
    MessageSent(Message),
}

#[derive(Debug)]
pub enum PostType {
    MetaEvent,
    Response,
    Message,
    MessageSent,
    Notice,
    Request,
    Unknown(String),
}

#[derive(Debug)]
pub enum MetaEvent {
    Lifecycle(Lifecycle),
    Heartbeat(Heartbeat),
}

/**
 * {
    "time": 1756177908,
    "self_id": 3775525519,
    "post_type": "meta_event",
    "meta_event_type": "heartbeat",
    "status": {
        "online": true,
        "good": true
    },
    "interval": 60000
}
 */
#[derive(Debug)]
pub struct Heartbeat {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub status: HeartbeatStatus,
    pub interval: i64,
}

#[derive(Debug)]
pub struct HeartbeatStatus {
    pub online: bool,
    pub good: bool,
}

#[derive(Debug)]
pub struct Lifecycle {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub sub_type: LifecycleSubType,
}

#[derive(Debug)]
pub enum LifecycleSubType {
    Enable,
    Disable,
    Connect,
}

/*
{
    "status": "ok",
    "retcode": 0,
    "data": {
        "message_id": 1936658337
    },
    "message": "",
    "wording": "",
    "echo": "b66ec140-7b5c-41fe-9422-b1ee89e4f89b"
}
 */
#[derive(Debug)]
pub struct Response {
    pub status: String,
    pub retcode: i64,
    pub data: serde_json::Value,
    pub message: String,
    pub wording: String,
    pub echo: String,
}

#[derive(Debug)]
pub struct Request {}

/*
{
    "self_id": 3775525519,
    "user_id": 815398013,
    "time": 1756168635,
    "message_id": 799899884,
    "message_seq": 92,
    "message_type": "private",
    "sender": {
        "user_id": 815398013,
        "nickname": "　",
        "card": ""
    },
    "raw_message": "33",
    "font": 14,
    "sub_type": "friend",
    "message": [
        {
            "type": "text",
            "data": {
                "text": "33"
            }
        }
    ],
    "message_format": "array",
    "post_type": "message"
}

 {
    "self_id": 3775525519,
    "user_id": 815398013,
    "time": 1756168837,
    "message_id": 70440311,
    "message_seq": 382,
    "message_type": "group",
    "sender": {
        "user_id": 815398013,
        "nickname": "　",
        "card": "",
        "role": "owner",
        "title": ""
    },
    "raw_message": "3",
    "font": 14,
    "sub_type": "normal",
    "message": [
        {
            "type": "text",
            "data": {
                "text": "3"
            }
        }
    ],
    "message_format": "array",
    "post_type": "message",
    "group_id": 559307734
}
 */
#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub enum MessageType {
    Private,
    Group,
}

#[derive(Debug, Clone)]
pub enum MessageFormat {
    Array,
    String,
}

#[derive(Debug, Clone)]
pub enum MessageSubType {
    Friend,
    Normal,
}

#[derive(Debug, Clone)]
pub struct Sender {
    pub user_id: i64,
    pub nickname: String,
    pub card: String,
    pub role: String,
    pub title: String,
}

/*

{
    "type": "text",
    "data": {
        "text": "33"
    }
}

    --

           {
            "type": "face",
            "data": {
                "id": "187",
                "sub_type": 1
            }
        }

    --

                {
            "type": "face",
            "data": {
                "id": "338",
                "sub_type": 3
            }
        }

        --

                {
            "type": "image",
            "data": {
                "file": "06B5164667AECA07BE7063487B0FE8BB.png",
                "subType": 2,
                "url": "https://multimedia.nt.qq.com.cn/download?appid=1407&fileid=EhQkEL7z20P2bcAws5tH_v8HMVVq5Ri5shUg_woo0oCVwKSnjwMyBHByb2RQgL2jAVoQQ0v9fc3pZu8zoidDsHeF9HoC1FY&spec=0&rkey=CAESME9sDrVVVyzMoGT09PHsf09Au0D248Da4C-M8_6RHrlp1glVCGxDFtNX1shylyxopg",
                "file_size": "350521"
            }
        }
        --

                {
            "type": "image",
            "data": {
                "file": "E98410F4B2AB990BB6285288F6896228.png",
                "subType": 1,
                "url": "https://multimedia.nt.qq.com.cn/download?appid=1407&fileid=EhTKREuLvwZvTt2YuLCnuoFStVCWbRjf8gkg_wookfmwzKSnjwMyBHByb2RQgL2jAVoQjyBWSYBnSNDxCu2VtwkXFHoCOgA&spec=0&rkey=CAESME9sDrVVVyzMoGT09PHsf09Au0D248Da4C-M8_6RHrlp1glVCGxDFtNX1shylyxopg",
                "file_size": "162143"
            }
        }

        --
        {
            "type": "at",
            "data": {
                "qq": "3775525519",
                "name": "gg"
            }
        }

        --

                {
            "type": "reply",
            "data": {
                "id": "1481434866"
            }
        },
 */
#[derive(Debug, Clone)]
pub enum MessageData {
    Text(MessageText),
    Face(MessageFace),
    Image(MessageImage),
    At(MessageAt),
    Reply(MessageReply),
    Forward(MessageForward),
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
            MessageData::Unknown(value) => value.clone().serialize(serializer),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFace {
    pub id: String,
    pub sub_type: i64,
}

impl Into<MessageData> for MessageFace {
    fn into(self) -> MessageData {
        MessageData::Face(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageImage {
    pub file: String,
    pub sub_type: i64,
    pub url: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAt {
    pub qq: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReply {
    pub id: i64,
}

impl Into<MessageData> for MessageReply {
    fn into(self) -> MessageData {
        MessageData::Reply(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageForward {
    pub id: String,
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

#[derive(Debug)]
pub enum Notice {
    // group_upload  group_admin group_decrease group_increase  group_ban friend_add group_recall friend_recall notify
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct GroupUpload {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub group_id: i64,
    pub user_id: i64,
    pub file: GroupUploadFile,
}

#[derive(Debug)]
pub struct GroupUploadFile {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub busid: i64,
}

#[derive(Debug)]
pub struct GroupAdmin {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: GroupAdminSubType,
    pub group_id: i64,
    pub user_id: i64,
}

#[derive(Debug)]
pub enum GroupAdminSubType {
    Set,
    UnSet,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum GroupDecreaseSubType {
    Leave,
    Kick,
    KickMe,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum GroupIncreaseSubType {
    Approve,
    Invite,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum GroupBanSubType {
    Ban,
    LiftBan,
}

#[derive(Debug)]
pub struct FriendAdd {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub user_id: i64,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct FriendRecall {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub user_id: i64,
    pub message_id: i64,
}

#[derive(Debug)]
pub enum Notify {
    Poke(Poke),
    LuckyKing(LuckyKing),
    Honor(Honor),
    Unknown(serde_json::Value),
}

#[derive(Debug)]
pub enum NotifySubType {
    Poke,
    LuckyKing,
    Honor,
    Unknown(String),
}

#[derive(Debug)]
pub struct Poke {
    pub time: i64,
    pub self_id: i64,
    pub post_type: PostType,
    pub notice_type: NoticeType,
    pub sub_type: NotifySubType,
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub enum HonorType {
    Talkative,
    Performer,
    Emotion,
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
            _ => Err(Error::FieldError("unknown post_type".to_string())),
        }
    }
}

impl PostType {
    pub fn parse(value: &str) -> Result<PostType> {
        match value {
            "meta_event" => Ok(PostType::MetaEvent),
            "message" => Ok(PostType::Message),
            "notice" => Ok(PostType::Notice),
            "request" => Ok(PostType::Request),
            "message_sent" => Ok(PostType::MessageSent),
            r#type => Ok(PostType::Unknown(r#type.to_string())),
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
        let message_format = message_format
            .as_str()
            .ok_or(Error::FieldError("message_format not found".to_string()))?;
        let message_format = MessageFormat::parse(message_format)?;
        let post_type = value
            .get("post_type")
            .ok_or(Error::FieldError("post_type not found".to_string()))?;
        let post_type = post_type
            .as_str()
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

impl Sender {
    pub fn parse(value: &serde_json::Value) -> Result<Sender> {
        let user_id = value
            .get("user_id")
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let user_id = user_id
            .as_i64()
            .ok_or(Error::FieldError("user_id not found".to_string()))?;
        let nickname = value
            .get("nickname")
            .ok_or(Error::FieldError("nickname not found".to_string()))?;
        let nickname = nickname
            .as_str()
            .ok_or(Error::FieldError("nickname not found".to_string()))?;
        let card = if let Some(card) = value.get("card") {
            card.as_str()
                .ok_or(Error::FieldError("card not found".to_string()))?
                .to_string()
        } else {
            "".to_string()
        };
        let role = if let Some(role) = value.get("role") {
            role.as_str()
                .ok_or(Error::FieldError("role not found".to_string()))?
                .to_string()
        } else {
            "".to_string()
        };
        let title = if let Some(title) = value.get("title") {
            title
                .as_str()
                .ok_or(Error::FieldError("title not found".to_string()))?
                .to_string()
        } else {
            "".to_string()
        };
        Ok(Sender {
            user_id,
            nickname: nickname.to_string(),
            card: card.to_string(),
            role: role.to_string(),
            title: title.to_string(),
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
        match r#type {
            "text" => Ok(MessageData::Text(MessageText::parse(value)?)),
            "face" => Ok(MessageData::Face(MessageFace::parse(value)?)),
            "image" => Ok(MessageData::Image(MessageImage::parse(value)?)),
            "at" => Ok(MessageData::At(MessageAt::parse(value)?)),
            "reply" => Ok(MessageData::Reply(MessageReply::parse(value)?)),
            "forward" => Ok(MessageData::Forward(MessageForward::parse(value)?)),
            _ => Ok(MessageData::Unknown(value.clone())),
        }
    }
}

impl MessageText {
    pub fn parse(value: &serde_json::Value) -> Result<MessageText> {
        let text = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let text = text
            .get("text")
            .ok_or(Error::FieldError("text not found".to_string()))?;
        let text = text
            .as_str()
            .ok_or(Error::FieldError("text not found".to_string()))?;
        Ok(MessageText {
            text: text.to_string(),
        })
    }
}

impl MessageFace {
    pub fn parse(value: &serde_json::Value) -> Result<MessageFace> {
        let id = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let id = id
            .get("id")
            .ok_or(Error::FieldError("id not found".to_string()))?;
        let id = id
            .as_str()
            .ok_or(Error::FieldError("id not found".to_string()))?;
        let sub_type = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let sub_type = sub_type
            .get("sub_type")
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        let sub_type = sub_type
            .as_i64()
            .ok_or(Error::FieldError("sub_type not found".to_string()))?;
        Ok(MessageFace {
            id: id.to_string(),
            sub_type,
        })
    }
}

impl MessageImage {
    pub fn parse(value: &serde_json::Value) -> Result<MessageImage> {
        let file = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let file = file
            .as_str()
            .ok_or(Error::FieldError("file not found".to_string()))?;
        let sub_type = if let Some(sub_type) = value.get("data") {
            sub_type
                .as_i64()
                .ok_or(Error::FieldError("sub_type not found".to_string()))?
        } else {
            0
        };
        let url = if let Some(url) = value.get("url") {
            url.as_str()
                .ok_or(Error::FieldError("url not found".to_string()))?
                .to_string()
        } else {
            "".to_string()
        };
        let file_size = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let file_size = file_size
            .get("file_size")
            .ok_or(Error::FieldError("file_size not found".to_string()))?;
        let file_size = file_size
            .as_i64()
            .ok_or(Error::FieldError("file_size not found".to_string()))?;
        Ok(MessageImage {
            file: file.to_string(),
            sub_type,
            url,
            file_size,
        })
    }
}

impl MessageAt {
    pub fn parse(value: &serde_json::Value) -> Result<MessageAt> {
        let data = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let qq = data
            .get("qq")
            .ok_or(Error::FieldError("qq not found".to_string()))?;
        let qq = qq
            .as_str()
            .ok_or(Error::FieldError("qq not found".to_string()))?;
        let name = if let Some(name) = data.get("name") {
            name.as_str()
                .ok_or(Error::FieldError("name not found".to_string()))?
                .to_string()
        } else {
            "".to_string()
        };
        Ok(MessageAt {
            qq: qq.to_string(),
            name: name.to_string(),
        })
    }
}

impl MessageReply {
    pub fn parse(value: &serde_json::Value) -> Result<MessageReply> {
        let id = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let id = id
            .get("id")
            .ok_or(Error::FieldError("id not found".to_string()))?;
        let id = id
            .as_i64()
            .ok_or(Error::FieldError("id not found".to_string()))?;
        Ok(MessageReply { id })
    }
}

impl MessageForward {
    pub fn parse(value: &serde_json::Value) -> Result<MessageForward> {
        let data = value
            .get("data")
            .ok_or(Error::FieldError("data not found".to_string()))?;
        let id = data
            .get("id")
            .ok_or(Error::FieldError("id not found".to_string()))?;
        let id = id
            .as_str()
            .ok_or(Error::FieldError("id not found".to_string()))?;
        Ok(MessageForward { id: id.to_string() })
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

impl MessageType {
    pub fn parse(value: &str) -> Result<MessageType> {
        match value {
            "private" => Ok(MessageType::Private),
            "group" => Ok(MessageType::Group),
            _ => Err(Error::FieldError("unknown message_type".to_string())),
        }
    }
}

impl MessageFormat {
    pub fn parse(value: &str) -> Result<MessageFormat> {
        match value {
            "array" => Ok(MessageFormat::Array),
            "string" => Ok(MessageFormat::String),
            _ => Err(Error::FieldError("unknown message_format".to_string())),
        }
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
        let message_format = message_format
            .as_str()
            .ok_or(Error::FieldError("message_format not found".to_string()))?;
        let message_format = MessageFormat::parse(message_format)?;
        let message_type = value
            .get("message_type")
            .ok_or(Error::FieldError("message_type not found".to_string()))?;
        let message_type = message_type
            .as_str()
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

impl Poke {
    pub fn parse(value: &serde_json::Value) -> Result<Poke> {
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
        let sub_type = NotifySubType::Poke;
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
        Ok(Poke {
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

impl Request {
    pub fn parse(_value: &serde_json::Value) -> Result<Request> {
        return Err(Error::FieldError("request unimplemented".to_string()));
    }
}
