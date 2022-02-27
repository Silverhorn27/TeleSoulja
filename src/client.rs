use std::ops::{Deref, DerefMut};

use anyhow::{Context, Result};

use grammers_client::SignInError;
use grammers_tl_types as tl_types;

use crate::utils::prompt;

#[derive(Clone)]
pub struct Client {
    inner: grammers_client::Client,
    sign_out: bool,
}

impl Drop for Client {
    fn drop(&mut self) {
        if self.sign_out {
            drop(futures::executor::block_on(
                self.inner.sign_out_disconnect(),
            ));
        }
    }
}

impl Client {
    pub async fn connect(api_id: i32, api_hash: &str, session: &str) -> Result<Self> {
        let config = grammers_client::Config {
            session: grammers_session::Session::load_file_or_create(session)?,
            api_id,
            api_hash: api_hash.to_string(),
            params: Default::default(),
        };

        log::info!("Connecting to Telegram...");
        let mut inner = grammers_client::Client::connect(config).await?;
        log::info!("Connected!");

        let mut sign_out = false;
        if !inner.is_authorized().await? {
            sign_in(&mut inner, api_id, &api_hash, session, &mut sign_out).await?;
        } else {
            log::debug!("The client is authorized");
        }

        Ok(Self { inner, sign_out })
    }

    #[allow(unused)]
    pub async fn join_channel(&self, username_or_link: &str) -> Result<tl_types::enums::Updates> {
        let resolved_join_type = ResolvedJoinType::resolve(username_or_link);
        match resolved_join_type {
            ResolvedJoinType::Username(username) => {
                let mut resolved_peer = self.resolve_username(username).await?;
                let chat = resolved_peer.chats.remove(0).into();
                if let tl_types::enums::Chat::Channel(channel) = chat {
                    let join_channel = tl_types::functions::channels::JoinChannel {
                        channel: tl_types::enums::InputChannel::Channel(
                            tl_types::types::InputChannel {
                                channel_id: channel.id,
                                access_hash: channel.access_hash.unwrap_or_default(),
                            },
                        ),
                    };
                    let updates = self.inner.invoke(&join_channel).await.unwrap();
                    Ok(updates.into())
                } else {
                    Err(anyhow::anyhow!("The used username is not a channel"))
                }
            }
            ResolvedJoinType::Hash(_) => todo!(),
        }
    }

    #[allow(unused)]
    pub async fn leave_channel(&self, username_or_link: &str) -> Result<tl_types::enums::Updates> {
        let resolved_join_type = ResolvedJoinType::resolve(username_or_link);
        match resolved_join_type {
            ResolvedJoinType::Username(username) => {
                let mut resolved_peer = self.resolve_username(username).await?;
                let chat = resolved_peer.chats.remove(0).into();
                if let tl_types::enums::Chat::Channel(channel) = chat {
                    let leave_channel = tl_types::functions::channels::LeaveChannel {
                        channel: tl_types::enums::InputChannel::Channel(
                            tl_types::types::InputChannel {
                                channel_id: channel.id,
                                access_hash: channel.access_hash.unwrap_or_default(),
                            },
                        ),
                    };
                    let updates = self.inner.invoke(&leave_channel).await.unwrap();
                    Ok(updates.into())
                } else {
                    Err(anyhow::anyhow!("The used username is not a channel"))
                }
            }
            ResolvedJoinType::Hash(_) => todo!(),
        }
    }

    #[allow(unused)]
    pub async fn get_full_channel(
        &self,
        username_or_link: &str,
    ) -> Result<tl_types::types::messages::ChatFull> {
        let resolved_join_type = ResolvedJoinType::resolve(username_or_link);
        match resolved_join_type {
            ResolvedJoinType::Username(username) => {
                let mut resolved_peer = self.resolve_username(username).await?;
                let chat = resolved_peer.chats.remove(0).into();
                if let tl_types::enums::Chat::Channel(channel) = chat {
                    let get_full_channel = tl_types::functions::channels::GetFullChannel {
                        channel: tl_types::enums::InputChannel::Channel(
                            tl_types::types::InputChannel {
                                channel_id: channel.id,
                                access_hash: channel.access_hash.unwrap_or_default(),
                            },
                        ),
                    };
                    let chat_full = self.inner.invoke(&get_full_channel).await.unwrap();
                    match chat_full {
                        tl_types::enums::messages::ChatFull::Full(full) => Ok(full),
                    }
                } else {
                    Err(anyhow::anyhow!("The used username is not a channel"))
                }
            }
            ResolvedJoinType::Hash(_) => todo!(),
        }
    }

    #[allow(unused)]
    pub async fn get_history(
        &self,
        username_or_link: &str,
    ) -> Result<tl_types::enums::messages::Messages> {
        let resolved_join_type = ResolvedJoinType::resolve(username_or_link);

        match resolved_join_type {
            ResolvedJoinType::Username(username) => {
                let mut resolved_peer = self.resolve_username(username).await?;
                let chat = resolved_peer.chats.remove(0).into();
                if let tl_types::enums::Chat::Channel(channel) = chat {
                    let get_full_channel = tl_types::functions::messages::GetHistory {
                        peer: tl_types::enums::InputPeer::Channel(
                            tl_types::types::InputPeerChannel {
                                channel_id: channel.id,
                                access_hash: channel.access_hash.unwrap_or_default(),
                            },
                        ),
                        offset_id: 0,
                        offset_date: 0,
                        add_offset: 2,
                        limit: 1,
                        max_id: i32::MAX,
                        min_id: i32::MIN,
                        hash: 0,
                    };
                    let messages = dbg!(self.inner.invoke(&get_full_channel).await.unwrap());
                    Ok(messages.into())
                } else {
                    Err(anyhow::anyhow!("The used username is not a channel"))
                }
            }
            ResolvedJoinType::Hash(_) => todo!(),
        }
    }

    pub async fn report_channel(&self, username_or_link: &str, message: String) -> Result<bool> {
        let resolved_join_type = ResolvedJoinType::resolve(username_or_link);

        match resolved_join_type {
            ResolvedJoinType::Username(username) => {
                let mut resolved_peer = match self.resolve_username(username).await {
                    Ok(peer) => peer,
                    Err(err) => {
                        log::error!("{}", err);
                        return Ok(false);
                    }
                };
                if resolved_peer.chats.len() < 1 {
                    return Ok(false);
                }
                let chat = resolved_peer.chats.remove(0).into();
                if let tl_types::enums::Chat::Channel(channel) = chat {
                    let peer =
                        tl_types::enums::InputPeer::Channel(tl_types::types::InputPeerChannel {
                            channel_id: channel.id,
                            access_hash: channel.access_hash.unwrap_or_default(),
                        });
                    let get_history = tl_types::functions::messages::GetHistory {
                        peer: tl_types::enums::InputPeer::Channel(
                            tl_types::types::InputPeerChannel {
                                channel_id: channel.id,
                                access_hash: channel.access_hash.unwrap_or_default(),
                            },
                        ),
                        offset_id: 0,
                        offset_date: 0,
                        add_offset: 0,
                        limit: 1,
                        max_id: i32::MAX,
                        min_id: i32::MIN,
                        hash: 0,
                    };
                    let messages = self.inner.invoke(&get_history).await.unwrap();
                    if let tl_types::enums::messages::Messages::ChannelMessages(messages) = messages
                    {
                        let id = messages.messages[0].id();
                        let reason = tl_types::enums::ReportReason::InputReportReasonViolence;
                        let report = tl_types::functions::messages::Report {
                            peer,
                            id: vec![id],
                            reason,
                            message,
                        };
                        return Ok(self.inner.invoke(&report).await.unwrap());
                    }
                }
            }
            ResolvedJoinType::Hash(_) => todo!(),
        }

        Ok(false)
    }

    async fn resolve_username(
        &self,
        username: String,
    ) -> Result<tl_types::types::contacts::ResolvedPeer> {
        let resolved_peer = self
            .inner
            .invoke(&tl_types::functions::contacts::ResolveUsername { username })
            .await
            .with_context(|| "Failed to resolve username")?;
        Ok(resolved_peer.into())
    }
}

impl Deref for Client {
    type Target = grammers_client::Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl AsRef<grammers_client::Client> for Client {
    fn as_ref(&self) -> &grammers_client::Client {
        &self.inner
    }
}

enum ResolvedJoinType {
    Username(String),
    Hash(String),
}

impl ResolvedJoinType {
    fn resolve(input: &str) -> Self {
        if let Some(username) = input.strip_prefix("@") {
            Self::Username(username.to_string())
        } else {
            let splitted = input.split("t.me/").last().unwrap_or_default();
            let parts: Vec<_> = splitted.split("/").collect();
            if parts.len() > 1 {
                Self::Hash(parts[1].to_string())
            } else {
                Self::Username(parts[0].to_string())
            }
        }
    }
}

async fn sign_in(
    client: &mut grammers_client::Client,
    api_id: i32,
    api_hash: &str,
    session: &str,
    sign_out: &mut bool,
) -> Result<()> {
    log::info!("Signing in...");
    let phone = prompt("Enter your phone number (international format): ")?;
    let token = client.request_login_code(&phone, api_id, &api_hash).await?;
    let code = prompt("Enter the code you received: ")?;
    let signed_in = client.sign_in(&token, &code).await;
    match signed_in {
        Err(SignInError::PasswordRequired(password_token)) => {
            // Note: this `prompt` method will echo the password in the console.
            //       Real code might want to use a better way to handle this.
            let hint = password_token.hint().unwrap();
            let prompt_message = format!("Enter the password (hint {}): ", &hint);
            let password = prompt(prompt_message.as_str())?;

            client
                .check_password(password_token, password.trim())
                .await?;
        }
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };
    log::info!("Signed in!");

    match client.session().save_to_file(session) {
        Ok(_) => {}
        Err(e) => {
            log::error!(
                "NOTE: failed to save the session, will sign out when done: {}",
                e
            );
            *sign_out = true;
        }
    }

    Ok(())
}
