# Valentine's Event for You-Zitsu (COTE) Server 2023

We made this discord bot for the Valentine's Event on the COTE Community Discord server. Members can send letters to another member or to their fictional heroine of the show. Select submissions will be read and reacted to on an episode of Podcast of the Elite.

## Requirements
- OS:
  - Windows
  - macOS (pre-compiled binaries are only x64 due to a [Github Action limitation](https://github.com/actions/runner-images/issues/2187), but we may provide compiled binaries from my own machine)
  - Linux

You will also need this environment file `.env` next to the executable or specify these in your CLI:

```env
# Specifies the location where the database will be saved
DATABASE_URL=sqlite.db

# Guild ID that you plan to use this in.
GUILD_ID=

# Your Discord Bot token from the Discord Developer Portal
DISCORD_TOKEN=

# (optional) Recipients to list in sendletter for autocomplete
RECIPIENTS=oralekin,Subject,Kiyotaka_Ayanokouji
```
After setting up this file you can run it by doing `./cotevalentines-Linux`, `./cotevalentines-macOS` (`chmod +x ./cotevalentines-*` may be required) or `& .\cotevalentines-Windows.exe` (Windows).

Use your background process manager of choice, be it `systemd`, Windows Services or `launchctl`, to launch it as a background service. 

It is recommended to use the docker image provided in this repository for production.

## Usage instructions (for the bot)

There are 3 commands available:
- `/sendletter recipient: String, letter: String, anonymous: Boolean` - accessible by everyone
- `/publish` - accessible by users with the Manage Messages permission
- `/add_recipient name: String, is_real: Boolean` - accessible by users with the Administrator permission

You can go to any channel where the bot is allowed or to the DMs of the bot and type `/sendletter`. 
You'll get prompted to enter a recipient, the contents of your letter and whether you want to send it anonymously.

- The minimum character count for your letter is 100
- It also won't send until you actually tell it whether you want it to be anonymous or not.
- After you're done typing in (or pasting) your letter, you can press enter to send it to the bot, where it will be stored in an SQLite Database.
  
Submitted letters will automatically get logged to a channel specified in your environment. 

By using the `/publis` command, the messages submitted by users will be published in the current channel with anonymity preserved.

## Compiling

Inside the project directory run `cargo build` for a debug build and `cargo build --all-features --release` for a release build.
The executable will be located in `./target/{debug|release}/cotevalentines` 