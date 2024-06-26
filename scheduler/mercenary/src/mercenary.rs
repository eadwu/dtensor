use futures_util::StreamExt;
use guild::Requirement;
use prost::Message;

#[derive(Clone)]
pub struct Mercenary {
    identifier: uuid::Uuid,
    client: async_nats::Client,
    capabilities: guild::Resources,
}

struct MercenaryChannel {
    topic: String,
    queue_group: String,
}

impl Mercenary {
    pub fn new(nc: async_nats::Client) -> Mercenary {
        let id = uuid::Uuid::new_v4();
        Mercenary {
            identifier: id,
            client: nc,
            capabilities: guild::Resources::default(),
        }
    }

    fn identifier(&self) -> String {
        self.identifier.to_string()
    }

    fn nats_client(&self) -> &async_nats::Client {
        &self.client
    }

    fn topic(&self) -> String {
        format!("{}.{}", guild::GUILD_MERCENARY_TOPIC, &self.identifier)
    }

    pub async fn routine(&self) -> Result<(), async_nats::Error> {
        tracing::info!(
            "Mercenary `{}` has been recruited for operations",
            &self.identifier
        );

        let communication_channels = [
            // Quest Board (broadcast)
            MercenaryChannel::new(guild::GUILD_QUEST_BOARD_TOPIC, guild::GUILD_DEFAULT_PARTY),
            // Direct Communication (unicast)
            MercenaryChannel::new(self.topic(), self.identifier()),
        ]
        .into_iter()
        .map(|channel| self.handler(channel.topic, channel.queue_group))
        .collect::<Vec<_>>();

        // Ignore Result handling here, instead make sure the async task never fails
        futures_util::future::join_all(communication_channels).await;
        Ok(())
    }

    async fn handler<T: Into<String>>(
        &self,
        topic: T,
        queue_group: T,
    ) -> Result<(), async_nats::Error> {
        let topic = topic.into();
        let queue_group = queue_group.into();

        tracing::debug!(
            "Mercenary `{}` is listening on board `{}` as part of party `{}`",
            &self.identifier,
            &topic,
            &queue_group,
        );

        let nats_client = self.nats_client();
        let mut subscription = nats_client
            .queue_subscribe(topic.clone(), queue_group.clone())
            .await?;

        while let Some(quest_msg) = subscription.next().await {
            let quest = guild::GuildQuest::decode(quest_msg.payload)?;
            let quest_identifier = quest.identifier;

            let satisfied_requirements = quest.requirements.map_or(true, |quest_requirements| {
                tracing::debug!(
                    "Quest `{}` requirements: {:?}",
                    &quest_identifier,
                    quest_requirements
                );
                self.capabilities.satisfies(&quest_requirements)
            });
            if satisfied_requirements {
                tracing::info!(
                    "Mercenary `{}` accepted quest `{}`",
                    &self.identifier,
                    &quest_identifier
                );
            } else {
                tracing::info!(
                    "Mercenary `{}` does not satisfy quest `{}` requirements",
                    &self.identifier,
                    &quest_identifier
                );
            }

            if let Some(reply_subject) = quest_msg.reply {
                tracing::debug!(
                    "Relaying status update of `{}` to `{}`",
                    &quest_identifier,
                    &reply_subject
                );

                let response = if satisfied_requirements {
                    guild::GuildQuestAcknowledgement::accept(quest_identifier, self.identifier())
                } else {
                    guild::GuildQuestAcknowledgement::deny(quest_identifier)
                };
                let payload = response.encode_to_vec();
                nats_client.publish(reply_subject, payload.into()).await?;
            }
        }

        tracing::debug!(
            "Mercenary `{}` is no longer actively looking at quest board `{}` as part of party `{}`",
            &self.identifier,
            &topic,
            &queue_group
        );
        Ok(())
    }
}

impl MercenaryChannel {
    fn new<T: Into<String>>(topic: T, queue_group: T) -> MercenaryChannel {
        MercenaryChannel {
            topic: topic.into(),
            queue_group: queue_group.into(),
        }
    }
}
