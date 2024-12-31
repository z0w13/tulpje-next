# Tulpje

Unnecessarily complicated rewrite of a multi-purpose discord bot

## Components

### Gateway

Receives [Gateway Events](https://discord.com/developers/docs/events/gateway-events) from discord and publishes them onto an AMQP queue.

Also handles storing shard statistics.

### Handler

The main "bot" component of Tulpje, this is where all the commands, event handlers, etc. live.

Works by connecting to an AMQP queue and listening for for Discord [Gateway Events](https://discord.com/developers/docs/events/gateway-events).

### Manager

Intended to be the component that manages (re)sharding, currently just returns
the recommended shard count from Discord.

### Framework

The bot framework used by `tulpje-handler` to handle events, commands, etc.

### Shared

Things shared between different parts of the bot.
