# A Tool for Exporting Your Baldur's Gate Mod Project files to a Separate Folder for Git Management

![alt text](<assets/2024-11-09 180440.png>)

## How To Use
- Set your Baldur's Gate Data Path, Just like you did in BG3 toolkit.
### Export
- When you export your mod project, you need to give the name of the mod(including the guid, Like "ThisIsAMod_29cec5fd-d5ce-15f1-b34f-88142900f99b").
- Create A Folder as you Destination Path.
- Select your export Destination Path, click export and this tool will move your mod's folders from BG3 Data path to your Destination Path. Softlinks are created in BG3 Data path.
- You can chose not to create default ignore and lfs config.
### Import
- When you import a mod, you just need to select the project where you export. It will fill the mod's name automatically.
- Click import and it will create softlinks for you.