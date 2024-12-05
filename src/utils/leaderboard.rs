use bson::Bson;
use chrono::Duration;
use serenity::UserId;
use tokio::time::sleep;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::prelude::*;

pub async fn leaderboards(ctx: &serenity::Context) -> Result<(), Error> {
    let sched = JobScheduler::new().await?;

    let ctx_clone = ctx.to_owned();

    sched
        .add(Job::new_async(
            std::env::var("LEADERBOARD").unwrap(),
            move |uuid, mut l| {
                let ctx_clone = ctx_clone.clone();
                Box::pin(async move {
                    log::debug!("Running leaderboard task");

                    if let Err(e) = handle_leaderboards(&ctx_clone).await {
                        println!("Error running handle_leaderboard: \n {e:#?}");
                    }

                    let next_tick = l.next_tick_for_job(uuid).await;
                    match next_tick {
                        Ok(Some(ts)) => log::debug!("Next run at {ts}"),
                        _ => log::error!("Error getting next time"),
                    }
                })
            },
        )?)
        .await?;

    sched.start().await?;

    loop {
        sleep(Duration::seconds(3600).to_std()?).await;
    }
}

async fn handle_leaderboards(ctx: &serenity::Context) -> Result<(), Error> {
    let guilds = GuildSetup::get_guilds().await?;

    for guild in guilds {
        let data = get_times_for_guild(guild.guild_id, ctx).await?;

        let mut table = ascii_table::AsciiTable::default();
        table
            .column(0)
            .set_header("User")
            .set_align(ascii_table::Align::Left);
        table
            .column(1)
            .set_header("Time in Voicechats")
            .set_align(ascii_table::Align::Right);

        let table_string = table.format(data);

        guild
            .channel_id
            .edit_message(
                ctx.http(),
                guild.leaderboard_message,
                serenity::EditMessage::default()
                    .content("Leaderboard: \n```".to_string() + &table_string + "\n```"),
            )
            .await?;
    }

    Ok(())
}

async fn get_times_for_guild(
    guild_id: serenity::GuildId,
    ctx: &serenity::Context,
) -> Result<Vec<Vec<String>>, Error> {
    let collection = UserDcEvent::get_collection().await;
    let two_weeks_ago =
        bson::DateTime::from_chrono(chrono::Utc::now() - chrono::Duration::weeks(2));
    let now = bson::DateTime::from_chrono(chrono::Utc::now());

    let mut cursor = collection.aggregate(vec![
            doc! {
                "$match": doc! {
                    "metadata.guild_id": guild_id.to_string(),
                }
            },
            doc! {
                "$sort": doc! {
                    "metadata.user_id": 1,
                    "timestamp": 1
                }
            },
            doc! {
                "$group": doc! {
                    "_id": "$metadata.user_id",
                    "events": doc! {
                        "$push": "$$ROOT"
                    }
                }
            },
            doc! {
                "$addFields": doc! {
                    "events": doc! {
                        "$cond": doc! {
                            "if": doc! {
                                "$eq": [{ "$arrayElemAt": ["$events.metadata.event", 0] }, UserEventType::Left]
                            },
                            "then": doc! {
                                "$concatArrays": [
                                    [
                                        {
                                            "timestamp": two_weeks_ago,
                                            "metadata": doc! {
                                                "event": UserEventType::Joined
                                            }
                                        }
                                    ],
                                    "$events"
                                ]
                            },
                            "else": "$events"
                        }
                    }
                }
            },
            doc! {
                "$addFields": doc! {
                    "events": doc! {
                        "$cond": doc! {
                            "if": doc! {
                                "$eq": [{ "$arrayElemAt": ["$events.metadata.event", -1] }, UserEventType::Joined]
                            },
                            "then": doc! {
                                "$concatArrays": [
                                    "$events",
                                    [
                                        {
                                            "timestamp": now,
                                            "metadata": doc! {
                                                "event": UserEventType::Left
                                            }
                                        }
                                    ],
                                ]
                            },
                            "else": "$events"
                        }
                    }
                }
            },
            doc! {
                "$project": doc! {
                    "_id": 1,
                    "joinedLeftPairs": doc! {
                        "$reduce": doc! {
                            "input": "$events",
                            "initialValue": doc! {
                                "joinedTimestamp": Bson::Null,
                                "pairs": []
                            },
                            "in": doc! {
                                "$cond": [
                                    doc! {
                                        "$eq": [
                                            "$$this.metadata.event",
                                            "Joined"
                                        ]
                                    },
                                    doc! {
                                        "joinedTimestamp": "$$this.timestamp",
                                        "pairs": "$$value.pairs"
                                    },
                                    doc! {
                                        "$cond": [
                                            doc! {
                                                "$and": [
                                                    doc! {
                                                        "$ne": [
                                                            "$$value.joinedTimestamp",
                                                            Bson::Null
                                                        ]
                                                    },
                                                    doc! {
                                                        "$eq": [
                                                            "$$this.metadata.event",
                                                            "Left"
                                                        ]
                                                    }
                                                ]
                                            },
                                            doc! {
                                                "joinedTimestamp": Bson::Null,
                                                "pairs": doc! {
                                                    "$concatArrays": [
                                                        "$$value.pairs",
                                                        [
                                                            doc! {
                                                                "joinedTimestamp": "$$value.joinedTimestamp",
                                                                "leftTimestamp": "$$this.timestamp"
                                                            }
                                                        ]
                                                    ]
                                                }
                                            },
                                            "$$value"
                                        ]
                                    }
                                ]
                            }
                        }
                    }
                }
            },
            doc! {
                "$unwind": "$joinedLeftPairs.pairs"
            },
            doc! {
                "$project": doc! {
                    "_id": 1,
                    "user_id": "$_id",
                    "duration": doc! {
                        "$subtract": [
                            "$joinedLeftPairs.pairs.leftTimestamp",
                            "$joinedLeftPairs.pairs.joinedTimestamp"
                        ]
                    }
                }
            },
            doc! {
                "$group": doc! {
                    "_id": "$_id",
                    "duration": doc! {
                        "$sum": "$duration"
                    }
                }
            },
            doc! {
                "$sort": doc! {
                    "duration": -1
                }
            },
            doc! {
                "$limit": 10
            }
        ]
    ).await?;

    let mut out = vec![];

    while let Some(user) = cursor.next().await {
        let user = user?;
        let user_id: UserId = user.get_str("_id")?.parse::<u64>()?.into();
        let user_name = user_id
            .to_user(ctx.http())
            .await?
            .display_name()
            .to_string();
        let duration = format_duration(std::time::Duration::from_millis(
            user.get_i64("duration")? as u64
        ));

        out.push(vec![user_name, duration]);
    }

    Ok(out)
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
