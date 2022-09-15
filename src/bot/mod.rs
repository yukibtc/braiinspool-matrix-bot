// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use braiinspool::Client as BraiinsPoolClient;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::{
    MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent, TextMessageEventContent,
};
use matrix_sdk::ruma::UserId;
use matrix_sdk::store::{CryptoStore, StateStore};
use matrix_sdk::{Client, ClientBuilder, Session};

mod autojoin;

use crate::{util, CONFIG, STORE};

pub struct Bot;

#[derive(Debug)]
pub enum Error {
    Db(bpns_rocksdb::Error),
    Matrix(matrix_sdk::Error),
    MatrixClientBuilder(matrix_sdk::ClientBuildError),
    MatrixStore(matrix_sdk::StoreError),
    MatrixCryptoStore(matrix_sdk::store::OpenStoreError),
    SlushPool(braiinspool::client::Error),
}

impl Bot {
    pub async fn run() -> Result<(), Error> {
        let homeserver_url: &str = CONFIG.matrix.homeserver_url.as_str();
        let user_id: &str = CONFIG.matrix.user_id.as_str();
        let password: &str = CONFIG.matrix.password.as_str();

        let user_id_boxed = Box::<UserId>::try_from(user_id).unwrap();
        let state_store = StateStore::open_with_path(&CONFIG.matrix.state_path)?;
        let crypto_store = CryptoStore::open_with_passphrase(&CONFIG.matrix.state_path, None)?;

        let mut client_builder: ClientBuilder = Client::builder()
            .homeserver_url(homeserver_url)
            .crypto_store(Box::new(crypto_store))
            .state_store(Box::new(state_store));

        if let Some(proxy) = &CONFIG.matrix.proxy {
            client_builder = client_builder.proxy(proxy);
        }

        let client: Client = client_builder.build().await?;

        log::debug!("Checking session...");

        if STORE.session_exist(user_id) {
            let session_store = STORE.get_session(user_id)?;

            let session = Session {
                access_token: session_store.access_token,
                user_id: user_id_boxed,
                device_id: session_store.device_id.into(),
            };

            client.restore_login(session).await?;

            log::debug!("Session restored from database");
        } else {
            log::debug!("Session not found into database");
            log::debug!("Login with credentials...");
            let username = user_id_boxed.localpart();
            client
                .login(username, password, None, Some("SlushPool Bot"))
                .await?;

            log::debug!("Getting session data...");

            if let Some(session) = client.session().await {
                log::debug!("Saving session data into database...");
                STORE.create_session(user_id, &session.access_token, session.device_id.as_ref())?;

                log::debug!("Session saved to database");
            } else {
                log::error!("Impossible to get and save session");
                log::warn!("The bot can continue to work without saving the session but if you are using an encrypted room, on the next restart, the bot will not be able to read the messages");
            }
        }

        client
            .account()
            .set_display_name(Some("BraiinsPool Bot"))
            .await?;

        log::info!("Matrix Bot started");

        client
            .register_event_handler(autojoin::on_stripped_state_member)
            .await
            .register_event_handler(
                move |event: OriginalSyncRoomMessageEvent, room: Room| async move {
                    if let Err(error) = Self::on_room_message(event, &room).await {
                        if let Room::Joined(room) = room {
                            let _ = room
                                .send(
                                    RoomMessageEventContent::text_plain(format!("{:?}", error)),
                                    None,
                                )
                                .await;
                        }
                    }
                },
            )
            .await;

        let settings = SyncSettings::default().full_state(true);
        client.sync(settings).await;

        Ok(())
    }

    async fn on_room_message(
        event: OriginalSyncRoomMessageEvent,
        room: &Room,
    ) -> Result<(), Error> {
        if *event.sender.clone() == CONFIG.matrix.user_id {
            return Ok(());
        }

        if let Room::Joined(room) = room {
            let msg_body = match event.content.msgtype {
                MessageType::Text(TextMessageEventContent { body, .. }) => body,
                _ => return Ok(()),
            };

            log::debug!("Message received: {}", msg_body);

            let start = Instant::now();

            let user_id: &str = event.sender.as_str();

            let proxy = CONFIG.proxy.as_deref();

            let msg_splitted: Vec<&str> = msg_body.split(' ').collect();
            let command: &str = msg_splitted[0];

            let mut msg_content: &str = "";

            match command {
                "!userstatus" => {
                    if STORE.user_exist(user_id) {
                        let user = STORE.get_user(user_id)?;

                        let client = BraiinsPoolClient::new(user.token.as_str(), proxy);

                        let obj = client.user_profile().await?;

                        let mut msg = String::from("User Status\n\n");
                        msg.push_str(&format!(
                            "Reward: {}\n",
                            util::format_btc_to_sats(obj.confirmed_reward)
                        ));
                        msg.push_str(&format!(
                            "Unconfirmed reward: {}\n",
                            util::format_btc_to_sats(obj.unconfirmed_reward)
                        ));
                        msg.push_str(&format!(
                            "Estimate reward (block): {}\n\n",
                            util::format_btc_to_sats(obj.estimated_reward)
                        ));

                        msg.push_str(&format!(
                            "Hashrate 5m: {}\n",
                            util::format_gh_to_th(obj.hash_rate_5m)
                        ));
                        msg.push_str(&format!(
                            "Hashrate 60m: {}\n",
                            util::format_gh_to_th(obj.hash_rate_60m)
                        ));
                        msg.push_str(&format!(
                            "Hashrate 24h: {}\n",
                            util::format_gh_to_th(obj.hash_rate_24h)
                        ));
                        msg.push_str(&format!(
                            "Hashrate scoring: {}\n",
                            util::format_gh_to_th(obj.hash_rate_scoring)
                        ));
                        msg.push_str(&format!(
                            "Hashrate yesterday: {}\n\n",
                            util::format_gh_to_th(obj.hash_rate_yesterday)
                        ));

                        msg.push_str(&format!("Ok workers: {}\n", obj.ok_workers));
                        msg.push_str(&format!("Low workers: {}\n", obj.low_workers));
                        msg.push_str(&format!("Off workers: {}\n", obj.off_workers));
                        msg.push_str(&format!("Disabled workers: {}", obj.dis_workers));

                        let content = RoomMessageEventContent::text_plain(msg);
                        room.send(content, None).await?;
                    } else {
                        msg_content = "This account in not subscribed.";
                    }
                }
                "!workers" => {
                    if STORE.user_exist(user_id) {
                        let user = STORE.get_user(user_id)?;

                        let client = BraiinsPoolClient::new(user.token.as_str(), proxy);

                        let obj = client.workers().await?;

                        let mut msg = String::from("Workers\n\n");

                        for (name, worker) in obj {
                            let name_splitted: Vec<&str> = name.split('.').collect();
                            if name_splitted.len() >= 2 {
                                msg.push_str(&format!("Worker: {}\n", name_splitted[1]));
                            }

                            msg.push_str(&format!("Status: {}\n", worker.state));
                            msg.push_str(&format!(
                                "Last share: {}\n",
                                util::format_date(worker.last_share as i64, "%Y-%m-%d %H:%M:%S")
                            ));
                            msg.push_str(&format!(
                                "Hashrate scoring: {}\n",
                                util::format_gh_to_th(worker.hash_rate_scoring)
                            ));
                            msg.push_str(&format!(
                                "Hashrate 5m: {}\n",
                                util::format_gh_to_th(worker.hash_rate_5m)
                            ));
                            msg.push_str(&format!(
                                "Hashrate 60m: {}\n",
                                util::format_gh_to_th(worker.hash_rate_60m)
                            ));
                            msg.push_str(&format!(
                                "Hashrate 24h: {}\n\n",
                                util::format_gh_to_th(worker.hash_rate_24h)
                            ));
                        }

                        let content = RoomMessageEventContent::text_plain(msg);
                        room.send(content, None).await?;
                    } else {
                        msg_content = "This account in not subscribed.";
                    }
                }
                "!dailyrewards" => {
                    if STORE.user_exist(user_id) {
                        let user = STORE.get_user(user_id)?;

                        let client = BraiinsPoolClient::new(user.token.as_str(), proxy);

                        let obj = client.daily_rewards().await?;

                        let mut msg = String::from("Daily Rewards\n\n");

                        for reward in obj {
                            msg.push_str(&format!(
                                "{}: {}\n",
                                util::format_date(reward.date as i64, "%Y-%m-%d"),
                                util::format_btc_to_sats(reward.total_reward)
                            ));
                        }

                        let content = RoomMessageEventContent::text_plain(msg);
                        room.send(content, None).await?;
                    } else {
                        msg_content = "This account in not subscribed.";
                    }
                }
                "!poolstatus" => {
                    if STORE.user_exist(user_id) {
                        let user = STORE.get_user(user_id)?;

                        let client = BraiinsPoolClient::new(user.token.as_str(), proxy);

                        let obj = client.pool_stats().await?;

                        let mut msg = String::from("Pool Status\n\n");
                        msg.push_str(&format!("Luck 10 blocks: {}\n", obj.luck_b10));
                        msg.push_str(&format!("Luck 50 blocks: {}\n", obj.luck_b50));
                        msg.push_str(&format!("Luck 250 blocks: {}\n", obj.luck_b250));
                        msg.push_str(&format!(
                            "Hashrate scoring: {}\n",
                            util::format_gh_to_th(obj.pool_scoring_hash_rate)
                        ));
                        msg.push_str(&format!(
                            "Active workers: {}\n",
                            util::format_number(obj.pool_active_workers as usize)
                        ));
                        msg.push_str(&format!("Round probability: {}\n", obj.round_probability));

                        let content = RoomMessageEventContent::text_plain(msg);
                        room.send(content, None).await?;
                    } else {
                        msg_content = "This account in not subscribed.";
                    }
                }
                "!subscribe" => {
                    let room_id: &str = room.room_id().as_str();

                    if !STORE.user_with_room_exist(user_id, room_id) {
                        if msg_splitted.len() >= 2 {
                            let token = msg_splitted[1];

                            if !token.is_empty() {
                                STORE.create_user(user_id, room_id, token)?;

                                let _ = room.redact(&event.event_id, None, None).await;
                                msg_content = "Subscribed";
                            } else {
                                msg_content =
                                "Please provide a token.\nTo subscribe send: !subscribe <token>";
                            }
                        } else {
                            msg_content =
                                "Please provide a token.\nTo subscribe send: !subscribe <token>";
                        }
                    } else {
                        msg_content = "This account is already subscribed";
                    }
                }
                "!unlink" => {
                    if STORE.user_exist(user_id) {
                        STORE.delete_user(user_id)?;
                        msg_content = "Unlinked";
                    } else {
                        msg_content = "No token linked to this account";
                    }
                }
                "!checktor" => {
                    let client = BraiinsPoolClient::new("", proxy);

                    let is_tor: bool = client.check_tor_connection().await?;

                    if is_tor {
                        msg_content = "Connected to Tor Network";
                    } else {
                        msg_content = "NOT connected to Tor Network";
                    }
                }
                "!help" => {
                    let mut msg = String::new();
                    msg.push_str("!userstatus - Get user status\n");
                    msg.push_str("!workers - Get workers\n");
                    msg.push_str("!dailyrewards - Get daily rewards\n");
                    msg.push_str("!poolstatus - Get pool status\n");
                    msg.push_str("!subscribe <token> - Subscribe with token\n");
                    msg.push_str("!unlink - Unlink account from token\n");
                    msg.push_str("!checktor - Check Tor connection\n");
                    msg.push_str("!help - Help");

                    let content = RoomMessageEventContent::text_plain(msg);
                    room.send(content, None).await?;
                }
                _ => {
                    msg_content = "Invalid command";
                }
            };

            if !msg_content.is_empty() {
                let content = RoomMessageEventContent::text_plain(msg_content);
                room.send(content, None).await?;
            }

            log::trace!(
                "{} command processed in {} ms",
                command,
                start.elapsed().as_millis()
            );
        }

        Ok(())
    }
}

impl From<bpns_rocksdb::Error> for Error {
    fn from(err: bpns_rocksdb::Error) -> Self {
        Error::Db(err)
    }
}

impl From<matrix_sdk::Error> for Error {
    fn from(err: matrix_sdk::Error) -> Self {
        Error::Matrix(err)
    }
}

impl From<matrix_sdk::ClientBuildError> for Error {
    fn from(err: matrix_sdk::ClientBuildError) -> Self {
        Error::MatrixClientBuilder(err)
    }
}

impl From<matrix_sdk::StoreError> for Error {
    fn from(err: matrix_sdk::StoreError) -> Self {
        Error::MatrixStore(err)
    }
}

impl From<matrix_sdk::store::OpenStoreError> for Error {
    fn from(err: matrix_sdk::store::OpenStoreError) -> Self {
        Error::MatrixCryptoStore(err)
    }
}

impl From<braiinspool::client::Error> for Error {
    fn from(err: braiinspool::client::Error) -> Self {
        Error::SlushPool(err)
    }
}
