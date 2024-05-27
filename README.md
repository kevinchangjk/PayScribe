![PayScribe Logo](./assets/logo-color.png)

# PayScribe

_Track your Telegram group payments right in the chat._

## What is PayScribe?

**PayScribe** is a Telegram bot designed to help you track your group expenses in any Telegram group chat. It does not require any signup, any registration, or any external tools. Just add the bot into your group chat, and you're ready to go!

## Table of Contents

1. [Features](#features)
2. [Getting Started as a User](#getting-started-as-a-user)\
   a. [Bot Commands](#bot-commands)\
   b. [User Guide](#user-guide)
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

### Bot Commands

`/start` — "Start" the bot.

`/help` — Show all commands and how to use the bot.

`/addpayment` — Add a new payment entry for the group.

`/payback` — Add a new entry paying back other members in the group.

`/viewpayments` — View all payment records for the group.

`/editpayment` — Edit a payment record that was previously added.

`/deletepayment` — Delete a payment record that was previously added.

`/balances` — View the current balances for the group.

`/spendings` — View the total spendings for the group.

`/settings` - View and edit bot settings for the group.

`/cancel` — Cancel an ongoing action.

### User Guide

For more details on how to use the bot, the various commands and configurations, do check out the [User Guide](https://payscribe.super.site/)!

The guide also contains more examples, tips, and advice on maximizing your use of **PayScribe**.

## For Developers

If you are interested, you are welcome to fork this repo and deploy your own bot.

This bot was written in Rust using [Teloxide](https://github.com/teloxide/teloxide), and uses a Redis database.

The API used for currency conversion rates is from [fawazahmed0](https://github.com/fawazahmed0/exchange-api).

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

- **Dispatcher**: Manages the conversation branches of the Telegram bot.
- **Processor**: Deals with the main backend facing logic for the bot, serves as intermediary between front-facing Handler and Redis.
- **Optimizer**: Separate crate for handling debt simplification logic, invoked by the Processor.
- **Currency**: Separate crate for handling currency-related logic, used by the Processor and Handler.
