# Available Apps & Plugins

This document lists all available applications that can be controlled via voice commands through the plugin system.

## Web Browsers

**Plugin:** `apps_browsers.json`

### Open Commands
- `open chrome` / `chrome` / `launch chrome`
- `open firefox` / `firefox` / `launch firefox`
- `open edge` / `edge` / `launch edge`
- `open safari` / `safari` / `launch safari`
- `open brave` / `brave` / `launch brave`
- `open opera` / `opera` / `launch opera`

### Close Commands
- `close chrome` / `quit chrome`
- `close firefox` / `quit firefox`
- `close edge` / `quit edge`
- `close safari` / `quit safari`
- `close brave` / `quit brave`
- `close opera` / `quit opera`

---

## Code Editors & IDEs

**Plugin:** `coders_plugin.json`

### Text Editors
- **Visual Studio Code**: `open vscode` / `open vs code` / `launch vscode` / `start coding` / `close vscode`
- **Sublime Text**: `open sublime` / `sublime text` / `launch sublime` / `close sublime`
- **Atom**: `open atom` / `launch atom` / `start atom` / `close atom`
- **Vim**: `open vim` / `launch vim` / `start vim` / `close vim`
- **Neovim**: `open neovim` / `nvim` / `launch neovim` / `close neovim`
- **Notepad++**: `open notepad++` / `notepad plus plus` / `npp` / `close notepad++`
- **Emacs**: `open emacs` / `launch emacs` / `close emacs`
- **Geany**: `open geany` / `launch geany` / `close geany`

### JetBrains IDEs
- **IntelliJ IDEA**: `open intellij` / `intellij idea` / `launch intellij` / `close intellij`
- **PyCharm**: `open pycharm` / `launch pycharm` / `python ide` / `close pycharm`
- **WebStorm**: `open webstorm` / `launch webstorm` / `web ide` / `close webstorm`
- **Rider**: `open rider` / `launch rider` / `csharp ide` / `close rider`
- **GoLand**: `open goland` / `launch goland` / `go ide` / `close goland`
- **PhpStorm**: `open phpstorm` / `launch phpstorm` / `php ide` / `close phpstorm`
- **RubyMine**: `open rubymine` / `launch rubymine` / `ruby ide` / `close rubymine`
- **CLion**: `open clion` / `launch clion` / `cpp ide` / `close clion`
- **DataGrip**: `open datagrip` / `launch datagrip` / `database ide` / `close datagrip`

### Other IDEs
- **Visual Studio**: `open visual studio` / `visual studio` / `launch visual studio` / `vs` / `close visual studio`
- **Eclipse**: `open eclipse` / `launch eclipse` / `java ide` / `close eclipse`

---

## System Utilities

**Plugin:** `apps_utilities.json`

### Open Commands
- `open calculator` / `calculator` / `calc`
- `open paint` / `paint` / `drawing app`
- `open terminal` / `terminal` / `open cmd`
- `open powershell` / `powershell` / `powershell terminal`

### Close Commands
- `close calculator` / `quit calculator`
- `close paint` / `quit paint`
- `close terminal` / `quit terminal`
- `close powershell` / `quit powershell`

---

## Communication & Collaboration

**Plugin:** `apps_communication.json`

### Open Commands
- `open discord` / `discord`
- `open slack` / `slack`
- `open teams` / `teams` / `microsoft teams`
- `open zoom` / `zoom`

### Close Commands
- `close discord` / `quit discord`
- `close slack` / `quit slack`
- `close teams` / `quit teams`
- `close zoom` / `quit zoom`

---

## Media & Entertainment

**Plugin:** `apps_media.json`

### Open Commands
- `open spotify` / `spotify` / `play music`

### Close Commands
- `close spotify` / `quit spotify`

---

## Office Productivity

**Plugin:** `apps_office.json`

### Open Commands
- `open word` / `word` / `microsoft word`
- `open excel` / `excel` / `spreadsheet`
- `open powerpoint` / `powerpoint` / `presentation`

### Close Commands
- `close word` / `quit word`
- `close excel` / `quit excel`
- `close powerpoint` / `quit powerpoint`

---

## Creative & Design

**Plugin:** `apps_creative.json`

### Open Commands
- `open photoshop` / `photoshop` / `image editor`
- `open illustrator` / `illustrator` / `vector editor`
- `open premiere` / `premiere` / `video editor`
- `open after effects` / `after effects` / `motion graphics`

### Close Commands
- `close photoshop` / `quit photoshop`
- `close illustrator` / `quit illustrator`
- `close premiere` / `quit premiere`
- `close after effects` / `quit after effects`

---

## Gaming Platforms

**Plugin:** `apps_gaming.json`

### Open Commands
- `open steam` / `steam` / `steam games`
- `open epic games` / `epic games` / `epic launcher`
- `open origin` / `origin` / `ea games`
- `open uplay` / `uplay` / `ubisoft`

### Close Commands
- `close steam` / `quit steam`
- `close epic games` / `quit epic`
- `close origin` / `quit origin`
- `close uplay` / `quit uplay`

---

## Example Plugins

**Plugin:** `example_plugin.json`

- `open youtube` - Opens YouTube in browser
- `check weather` / `show weather` - Opens weather website
- `open github` - Opens GitHub in browser

---

## Usage Notes

### Voice Command Format
- **Open Apps**: `"open [app name]"` or just the app name
  - Example: `"open chrome"` or `"chrome"`
  
- **Close Apps**: `"close [app name]"` or `"quit [app name]"`
  - Example: `"close chrome"` or `"quit chrome"`

### Multiple Browsers
The system now correctly distinguishes between different browsers. Each browser has its own trigger:
- Saying `"open chrome"` will open **only Chrome**
- Saying `"open firefox"` will open **only Firefox**
- NOT opening both browsers

### Command Matching
Commands are matched in priority order:
1. **Exact match** - Matches the exact trigger phrase
2. **Example match** - Matches any example variation
3. **Contains match** - Matches if the full trigger is contained in your command

### Example Commands
```
"open chrome"              → Opens Google Chrome
"launch firefox"           → Opens Mozilla Firefox
"open visual studio code"  → Opens VSCode
"close photoshop"          → Closes Adobe Photoshop
"quit slack"               → Closes Slack
"open calculator"          → Opens Calculator
"spotify"                  → Opens Spotify
```

---

## Adding New Apps

To add new apps to the system:

1. Create a new JSON file in the `plugins/` directory with the plugin structure:
   ```json
   {
     "metadata": {
       "name": "plugin_name",
       "version": "1.0.0",
       "author": "IGRIS",
       "description": "Description of the plugin",
       "keywords": ["keyword1", "keyword2"],
       "enabled": true
     },
     "commands": [
       {
         "trigger": "open appname",
         "description": "Opens the application",
         "examples": ["open appname", "appname"],
         "action_type": "ShellCommand",
         "action_data": "start appname.exe"
       },
       {
         "trigger": "close appname",
         "description": "Closes the application",
         "examples": ["close appname", "quit appname"],
         "action_type": "ShellCommand",
         "action_data": "taskkill /IM appname.exe /F"
       }
     ]
   }
   ```

2. The system will automatically load the new plugin on next startup
3. The NER engine will automatically extract app names and keywords for semantic matching

---

## System Information

- **Total Plugins**: 9
- **Total Apps**: 60+
- **Supported Platforms**: Windows (primary), Linux, macOS (partial)
- **Command Execution**: Real-time voice command processing
- **Plugin System**: Hot-loadable JSON-based configuration

---

## Troubleshooting

### App Not Opening
- Verify the executable name matches the Windows application name
- Check if the app is installed at the expected location
- Try running the command manually in PowerShell

### App Not Closing
- Ensure the executable name in the `taskkill` command is correct
- Some applications may require different process names to close

### Plugin Not Loading
- Verify JSON syntax is correct
- Check file permissions in the `plugins/` directory
- Ensure file extension is `.json`

---

Last Updated: January 1, 2026
