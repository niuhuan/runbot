pub mod can_send_image;
pub mod can_send_record;
pub mod delete_msg;
pub mod get_forward_msg;
pub mod get_friend_list;
pub mod get_group_info;
pub mod get_group_list;
pub mod get_group_member_info;
pub mod get_group_member_list;
pub mod get_image;
pub mod get_msg;
pub mod get_record;
pub mod send_message;
pub mod set_friend_add_request;
pub mod set_group_add_request;
pub mod set_group_admin;
pub mod set_group_ban;
pub mod set_group_card;
pub mod set_group_kick;
pub mod set_group_leave;
pub mod set_group_name;
pub mod set_group_special_title;
pub mod set_group_whole_ban;

pub mod create_group_file_folder;
pub mod delete_friend;
pub mod delete_group_file;
pub mod delete_group_folder;
pub mod forward_friend_single_msg;
pub mod forward_group_single_msg;
pub mod get_group_file_url;
pub mod get_group_files_by_folder;
pub mod get_group_root_files;
pub mod get_private_file_url;
pub mod mark_msg_as_read;
pub mod ocr_image;
pub mod rename_group_file_folder;
pub mod send_poke;
pub mod set_group_remark;
pub mod set_group_sign;
pub mod upload_group_file;

// 账号相关
pub mod create_favorite;
pub mod get_account_info;
pub mod get_favorite_face;
pub mod get_filtered_friend_requests;
pub mod get_like_list;
pub mod get_login_info;
pub mod get_miniapp_card;
pub mod get_online_clients;
pub mod get_online_model;
pub mod get_recent_contact;
pub mod get_recommended_friends;
pub mod get_recommended_groups;
pub mod get_status;
pub mod get_unidirectional_friend_list;
pub mod get_user_status;
pub mod get_friend_group_list;
pub mod handle_filtered_friend_request;
pub mod send_like;
pub mod set_account_profile;
pub mod set_all_msg_read;
pub mod set_avatar;
pub mod set_custom_online_status;
pub mod set_friend_remark;
pub mod set_group_msg_read;
pub mod set_online_model;
pub mod set_online_status;
pub mod set_private_msg_read;
pub mod set_signature;

// 消息相关
pub mod get_group_history_msg;
pub mod get_image_detail;
pub mod get_private_history_msg;
pub mod get_record_detail;
pub mod send_forward_msg;
pub mod send_group_ai_voice;
pub mod get_essence_msg_list;
pub mod set_essence_msg;

// 群聊相关
pub mod batch_kick_group_member;
pub mod delete_essence_msg;
pub mod delete_group_notice;
pub mod get_group_at_all_remain;
pub mod get_group_ban_list;
pub mod get_group_filter_system_msg;
pub mod get_group_honor;
pub mod get_group_info_ex;
pub mod get_group_notice;
pub mod get_group_system_msg;
pub mod group_check_in;
pub mod send_group_notice;
pub mod set_group_add_option;
pub mod set_group_avatar;
pub mod set_group_bot_add_option;
pub mod set_group_search;

// 文件相关
pub mod download_file_to_cache;
pub mod get_file_info;
pub mod get_group_file_system_info;
pub mod move_group_file;
pub mod rename_group_file;
pub mod save_file_to_cache;
pub mod upload_private_file;

// 密钥相关
pub mod get_clientkey;
pub mod get_cookies;
pub mod get_credentials;
pub mod get_csrf_token;
pub mod get_rkey;
pub mod get_rkey_service;
pub mod nc_get_rkey;

// 个人操作
pub mod click_button;
pub mod get_ai_voice;
pub mod get_ai_voice_person;
pub mod handle_quick_operation;
pub mod set_input_status;
pub mod translate_en_to_zh;

// 系统操作
pub mod account_logout;
pub mod clear_cache;
pub mod get_bot_account_range;
pub mod get_packet_status;
pub mod get_version_info;
pub mod send_custom_packet;
