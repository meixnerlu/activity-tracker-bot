# Activity Tracker Bot

A bot that tracks voice chat activity for all users or users with a specified role.
All data will be deleted after 2 weeks.
Displays a leaderboard with the top 10 most active users.

Build with [poise](https://github.com/serenity-rs/poise) and [mongodb](https://www.mongodb.com/)

Invite link: [discord.com](https://discord.com/oauth2/authorize?client_id=1282725155944140851)

## Bot Usage

Getting started:

<code>/setup run</code>

Starts a wizard like dialogue to get you started.
If you select a role, you can add a button for your users to self-assign the role:

<code>/setup role_button</code>

If you want to change the role selection you can use:

<code>/setup edit</code>

You can remove the bot from tracking your members and delete all data with this command:

<code>/setup delete</code>

## Self host

Build the docker image.

env:

| Varible | Value |
| -------------- | --------------- |
| MONGODB | full uri string to mongodb database/replicaset |
| DISCORD_TOKEN | Bot token from [discord.dev](https://discord.dev) |
| LEADERBOARD | Cron like timing for how often the leaderboards get updated [tokio-cron-scheduler](https://github.com/mvniekerk/tokio-cron-scheduler) |
