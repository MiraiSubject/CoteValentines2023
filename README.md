# Valentine's Event for You-Zitsu (COTE) Server 2023

We made this discord bot for the Valentine's Event on the COTE Community Discord server. Members can send letters to another member or to their fictional heroine of the show. 

## Requirements
- OS:
  - Windows
  - macOS (pre-compiled binaries are only x64 due to a [Github Action limitation](https://github.com/actions/runner-images/issues/2187), but we may provide compiled binaries from my own machine)
  - Linux

You will also need this environment file `.env` next to the executable or specify these in your CLI:

```
# Specifies the location where the database will be saved
DATABASE_URL=sqlite.db

# Guild ID that you plan to use this in.
GUILD_ID=

# Your Discord Bot token from the Discord Developer Portal
DISCORD_TOKEN=

# (optional) Recipients to list in sendletter for autocomplete
RECIPIENTS=oralekin,Subject,Kiyotaka_Ayanokouji
```
After setting up this file you can run it by doing `./Cotevalentines2023` (macOS/Linux) (`chmod +x ./Cotevalentines2023` may be required) or `& .\Cotevalentines2023.exe` (Windows)

It is recommended to use the docker image provided in this repository for production.

## Usage instructions (for the bot)

There are 3 commands available:
- `/sendletter recipient: String, letter: String, anonymous: Boolean` - accessible by everyone
- `/publish channel: Channel` - accessible by users with the Manage Messages permission
- `/add_recipient name: String, is_real: Boolean` - accessible by users with the Administrator permission

You can go to any channel where the bot is allowed or to the DMs of the bot and type `/sendletter`. 
You'll get prompted to enter a recipient, the contents of your letter and whether you want to send it anonymously.

- The minimum character count for your letter is 100
- It also won't send until you actually tell it whether you want it to be anonymous or not.
- After you're done typing in (or pasting) your letter, you can press enter to send it to the bot, where it will be stored in an SQLite Database.
  
Submitted letters will automatically get logged to a channel specified in your environment. 

By using the `/publish #channel` command, the messages submitted by users will be published in the specified channel with anonymity preserved.

## Compiling
