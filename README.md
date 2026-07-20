# asapi

`asapi` is a command-line client for Apple App Store search. It does not need any authentication.

It is intended to be used by AI agents to retrieve data from the App Store. To install the skill, run:

```bash
asapi install-skill
```
## Install or Update

On macOS or Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/maximbezd99/asapi-cli/main/install.sh | bash
```

The installer supports Intel/AMD and ARM systems and installs to `~/.local/bin` by default. To choose another directory:

```bash
curl -fsSL https://raw.githubusercontent.com/maximbezd99/asapi-cli/main/install.sh | INSTALL_DIR=<dir> bash
```