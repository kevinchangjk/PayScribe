# PayScribe

_Track your Telegram group payments right in the chat._

## What is PayScribe?

**PayScribe** is a Telegram bot designed to help you track your group expenses in any Telegram group chat. It does not require any signup, any registration, or any external tools. Just add the bot into your group chat, and you're ready to go!

## Table of Contents

1. [Features](#features)
2. [Getting Started as a User](#getting-started-as-a-user)\
   a. [Bot Commands](#bot-commands)
3. [For Developers](#for-developers)\
   a. [Setup](#setup)\
   b. [Codebase](#codebase)

## Features

- Tracking group payment records
- Automatic simplification of debts within groups
- Complete viewability and editability of all payment records
- 3 different modes of splitting the costs
  - By equal amounts
  - By exact amounts
  - By proportionate amounts
- **No setup required**, everything runs right within the chat

## Getting Started as a User

**PayScribe** can be found at [@PayScribeBot](https://t.me/PayScribeBot).

To get started with using the bot, add the bot into any group chat of your choice! You can then check out the available bot commands with `/help`, or as listed below.

Do note that by default, Telegram bots will have <a href="https://core.telegram.org/bots/features#:~:text=Privacy%20mode%20is%20enabled%20by,this%20change%20to%20take%20effect" target="_blank">Group Privacy</a> enabled, which means that they can only access messages that are directed towards them in some way. To interact with the **PayScribe**, users will have to call the bot's commands, or directly reply to the bot's messages.

If you notice that **PayScribe** is responding to every message sent in the group, that means that the bot has been given admin rights, and can thus access every message in the group. Revoke the bot's admin rights for normal usage.

### Bot Commands

`/start` — "Start" the bot.

`/help` — Show all commands and how to use the bot.

`/addpayment` — Add a new payment entry for the group.

`/payback` — Add a new entry paying back other members in the group to settle debts.

`/viewpayments` — View all payment records for the group.

`/editpayment` — Edit a payment record that was previously added.

`/deletepayment` — Delete a payment record that was previously added.

`/viewbalances` — View the current balances for the group.

`/cancel` — Cancels an ongoing action.

## For Developers

If you are interested, you are welcome to fork this repo and deploy your own bot.

This bot was written in Rust using [Teloxide](https://github.com/teloxide/teloxide), and uses a Redis database.

Below, I will go through the steps for setting up the environment, and an overview of the codebase.

### Setup

1. Clone the repo.

2. Ensure that you have Redis, Rust, and Cargo installed.

3. Add environment variables to a new file `.env`. You will need two variables:

   - `TELOXIDE_TOKEN`: API key for your Telegram bot, [get one from the BotFather](https://core.telegram.org/bots/tutorial)
   - `REDIS_URL`: URL for your Redis server, can be local

4. Start your Redis server, and run the following command in the project root directory:

```bash
cargo run
```

### Codebase

The codebase consists of mainly the **Bot** module, which has the following submodules:

- **Handler**: Deals with user-facing/UX logic, mainly the back-and-forth conversational logic for the bot. The Handler is invoked by the Dispatcher, and calls the Processor's functions.
- **Redis**: Contains all database-related CRUD operations. Exposes a set of functions for the Processor to call.

Apart from these, the other main components of the bot are:

- **Dispatcher**: Manages the conversation branches of the Telegram bot, and runs the server.
- **Processor**: Deals with the main backend facing logic for the bot, serves as intermediary between front-facing Handler and Redis.
- **Optimizer**: Separate crate for handling debt simplification logic, invoked by the Processor.
