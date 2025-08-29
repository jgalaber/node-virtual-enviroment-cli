# NVE — Node Version Environment

A blazing-fast, cross-platform Node.js version manager written in Rust — **zero admin privileges required**.

Perfect for enterprise environments like banks and corporations where software installations are restricted. NVE operates entirely within your user directory, making it ideal for environments with strict security policies.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform Support](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/jgalaber/node-virtual-enviroment-cli)

## ✨ Features

- 🚀 **Lightning Fast** — Single lightweight Rust binary with minimal overhead
- 🔒 **Zero Admin Rights** — Install and manage Node.js versions without elevated permissions
- 🖥 **Universal Support** — Windows, macOS, and Linux
- 🏢 **Enterprise Ready** — Perfect for restricted corporate environments
- 📦 **Complete Management** — Install, remove, list, and switch Node.js versions
- 🌍 **Smart Installer** — Automatic OS detection and setup
- 💼 **Isolated Environment** — All versions stored in user directory (`~/.nve`)

---

## 📋 Quick Start

Choose your operating system:

- [🪟 Windows Installation](#-windows-installation)
- [🍎 macOS Installation](#-macos-installation)
- [🐧 Linux Installation](#-linux-installation)

---

## 📥 Installation

### 🪟 Windows Installation

#### Option 1: PowerShell Installer (Recommended)

```powershell
# Run in PowerShell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/jgalaber/node-virtual-enviroment-cli/main/install.ps1" -OutFile "install.ps1"
PowerShell -ExecutionPolicy Bypass -File "install.ps1"
Remove-Item "install.ps1"
```

#### Option 2: Manual Installation

1. Download the Windows binary from [Releases](https://github.com/jgalaber/node-virtual-enviroment-cli/releases)
2. Create directory: `mkdir %USERPROFILE%\.nve\bin`
3. Move `nve.exe` to `%USERPROFILE%\.nve\bin\nve.exe`
4. Add `%USERPROFILE%\.nve\bin` to your PATH

---

### 🍎 macOS Installation

#### Option 1: Universal Installer (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/jgalaber/node-virtual-enviroment-cli/main/install.sh | bash
```

#### Option 3: Manual Installation

For Apple Silicon (M1/M2):

```bash
curl -L https://github.com/jgalaber/node-virtual-enviroment-cli/releases/download/v0.1.0/nve-aarch64-apple-darwin -o ~/.nve/bin/nve
chmod +x ~/.nve/bin/nve
```

For Intel Macs:

```bash
curl -L https://github.com/jgalaber/node-virtual-enviroment-cli/releases/download/v0.1.0/nve-x86_64-apple-darwin -o ~/.nve/bin/nve
chmod +x ~/.nve/bin/nve
```

---

### 🐧 Linux Installation

#### Option 1: Universal Installer (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/jgalaber/node-virtual-enviroment-cli/main/install.sh | bash
```

#### Option 2: Manual Installation

For x86_64:

```bash
curl -L https://github.com/jgalaber/node-virtual-enviroment-cli/releases/download/v0.1.0/nve-x86_64-unknown-linux-gnu -o ~/.nve/bin/nve
chmod +x ~/.nve/bin/nve
```

For ARM64:

```bash
curl -L https://github.com/jgalaber/node-virtual-enviroment-cli/releases/download/v0.1.0/nve-aarch64-unknown-linux-gnu -o ~/.nve/bin/nve
chmod +x ~/.nve/bin/nve
```

#### Option 3: Package Managers

**Ubuntu/Debian:**

```bash
wget https://github.com/jgalaber/node-virtual-enviroment-cli/releases/download/v0.1.0/nve_0.1.0_amd64.deb
sudo dpkg -i nve_0.1.0_amd64.deb
```

**CentOS/RHEL/Fedora:**

```bash
wget https://github.com/jgalaber/node-virtual-enviroment-cli/releases/download/v0.1.0/nve-0.1.0-1.x86_64.rpm
sudo rpm -i nve-0.1.0-1.x86_64.rpm
```

---

## ⚙️ Shell Configuration

### Windows (PowerShell)

Add to your PowerShell profile (`$PROFILE`):

```powershell
$env:NVE_HOME = "$env:USERPROFILE\.nve"
$env:PATH = "$env:NVE_HOME\current\bin;$env:PATH"
```

### Windows (Command Prompt)

Add to your system environment variables or create a batch file:

```batch
set NVE_HOME=%USERPROFILE%\.nve
set PATH=%NVE_HOME%\current\bin;%PATH%
```

### macOS/Linux (Bash)

Add to `~/.bashrc`:

```bash
export NVE_HOME="$HOME/.nve"
export PATH="$NVE_HOME/current/bin:$PATH"
```

### macOS/Linux (Zsh)

Add to `~/.zshrc`:

```bash
export NVE_HOME="$HOME/.nve"
export PATH="$NVE_HOME/current/bin:$PATH"
```

### macOS/Linux (Fish)

Add to `~/.config/fish/config.fish`:

```fish
set -gx NVE_HOME "$HOME/.nve"
set -gx PATH "$NVE_HOME/current/bin" $PATH
```

After configuration, reload your shell:

```bash
# Bash/Zsh
source ~/.bashrc  # or ~/.zshrc

# Fish
source ~/.config/fish/config.fish

# PowerShell
. $PROFILE
```

---

## 📚 Usage

### Install Node.js Versions

```bash
nve install 20.10.0      # Install specific version
nve install 20           # Install latest 20.x.x
nve install 18.17.1      # Install Node.js 18.17.1
nve install lts          # Install latest LTS version
nve install latest       # Install latest stable version

# Aliases: add
nve add 16.20.0
```

### Switch Between Versions

```bash
nve use 20.10.0          # Switch to Node.js 20.10.0
node -v                  # Verify: v20.10.0
npm -v                   # NPM version included with Node.js

nve use 18               # Switch to latest installed 18.x.x
```

### List Installed Versions

```bash
nve list                 # Show all locally installed versions
# Output:
# * 20.10.0 (current)
#   18.17.1
#   16.20.0

nve ls                   # Shorthand alias
```

### Remove Versions

```bash
nve remove 16.20.0       # Remove specific version
nve remove 18            # Remove all 18.x.x versions

# Aliases: uninstall, rm
nve uninstall 20.10.0
```

### Check Remote Versions

```bash
nve remote 20            # Get latest remote 20.x.x version
nve remote lts           # Get latest LTS version
nve remote latest        # Get latest stable version
```

### Additional Commands

```bash
nve current              # Show currently active version
nve which                # Show path to current Node.js binary
nve version              # Show NVE version
nve help                 # Show help information
```

---

## 💼 Enterprise Use Cases

NVE is specifically designed for corporate environments with restricted permissions:

### Banking & Financial Institutions

- Install Node.js without system privileges
- Maintain compliance with security policies
- Switch versions for different projects seamlessly

### Corporate Development Teams

- Each developer manages their own Node.js versions
- No conflicts with system-wide installations
- Easy onboarding for new team members

### CI/CD Environments

- Consistent Node.js versions across build pipelines
- Fast version switching in automated workflows
- No admin privileges required in containers

---

## 🚀 Quick Examples

### Project-Based Version Management

```bash
# Working on different projects
cd project-legacy
nve use 16            # Older project needs Node 16

cd ../project-modern  
nve use 20            # New project uses Node 20

cd ../project-experimental
nve install 21        # Try bleeding-edge features
nve use 21
```

### Team Development

[Work in progress...]

```bash
# Share exact versions with team
echo "20.10.0" > .nvmrc          # Create version file
nve use $(cat .nvmrc)            # Use version from file

# Or use NVE's auto-detection
nve use # Automatically uses .nvmrc or .nverc version
```

---

## 🔍 Troubleshooting

### Windows Issues

- **PATH not updated**: Restart PowerShell/Command Prompt after installation
- **Permission denied**: Ensure you have write access to `%USERPROFILE%\.nve`
- **Antivirus blocking**: Add NVE directory to antivirus exclusions

### macOS/Linux Issues

- **Command not found**: Verify `~/.nve/bin` is in your PATH
- **Permission denied**: Run `chmod +x ~/.nve/bin/nve`
- **Download fails**: Check internet connection and firewall settings

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📜 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Inspired by [nvm](https://github.com/nvm-sh/nvm), [fnm](https://github.com/Schniz/fnm), and [volta](https://github.com/volta-cli/volta)
- Built with [Rust](https://rust-lang.org/) for performance and reliability
- Node.js versions provided by [Node.js Foundation](https://nodejs.org)

---

## 📞 Support

- 📖 [Documentation](https://github.com/jgalaber/node-virtual-enviroment-cli/wiki)
- 🐛 [Issue Tracker](https://github.com/jgalaber/node-virtual-enviroment-cli/issues)
- 💬 [Discussions](https://github.com/jgalaber/node-virtual-enviroment-cli/discussions)
- 📧 [Email Support](mailto:jgalaberdev@gmail.com)
