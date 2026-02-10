# GameSaveSync

A tool to synchronize game saves between multiple devices, so that you can play a game across multiple devices without needing to think about where you played last. Ideally, the experience should be similar to Steam Cloud or other platform saves. This project is in very early development, so expect missing features and potentially corrupted saves.

## Planned Usage

1. `gamesavesync set-repository LOCATION`

Set the location for game saves to be stored, expected to be accessible by all devices

2. `gamesavesync register GAME`

Add a new game entry to the repository, specifying where saves are found. Will also link the game by default.

3. `gamesavesync link GAME`

Activate save sync for the game on this device. Will attempt an initial sync, as the device save is likely be in conflict with the repository.

4. `gamesavesync sync`

Perform a save sync, if the local save or repository have changed the other will be updated to match. If both have changed the user will be prompted to choose one of the two or to do nothing.

The sync command might get called before and after the game by inserting it into the shortcut or Steam launch options. Prompting the user if there is a conflict needs to be investigated.

## Planned Features

- Support remote save repository locations such as network shares
- Support for saves in files/folders and registry keys
- Normalize paths and registry keys between windows and proton
- Identify save conflicts if local saves and the repository have both changed
- Lots of error checking and logging for conflicts
- Use the [Ludusavi Manifest](https://github.com/mtkennerly/ludusavi-manifest) for easy game registration

## Potential Features

- A nice GUI with all of the CLI functionality
- Synchronize save files/folders across windows-native and linux-native builds of a game - Will need investigation on how to identify compatible paths
- Come up with a more interesting name

## Not Planned Features

- Automatic sync before/after playing a game - I don't want this tool to have to run in the background and monitor processes, other monitoring tools exist which could call the sync command