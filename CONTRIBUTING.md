# Contributing to NVE

First off, thank you for considering contributing to **NVE (Node Version Environment)**! 🎉  
This document outlines the process for contributing, our coding guidelines, and best practices.

---

## 🛠 Getting Started

### 1. Fork and Clone the Repository

```bash
git clone https://github.com/jgalaber/node-virtual-enviroment-cli
cd node-virtual-enviroment-cli
```

### 2. Install Rust

Make sure you have Rust installed (latest stable version recommended).

Check:

```bash
rustc --version
cargo --version
```

### 3. Install Dependencies

NVE uses Cargo workspaces. All dependencies are managed automatically:

```bash
cargo check
```

### 4. Enable Git Hooks

We use **native Git hooks** to run `cargo fmt`, `cargo clippy`, and `cargo test` automatically before commits and pushes.

Run once after cloning:

```bash
./scripts/setup-git-hooks.sh
```

This will configure Git to use the versioned hooks in `.githooks/`.

You can verify:

```bash
git config core.hooksPath   # should show .githooks
```

### 5. Run Locally

To build and run the CLI:

```bash
cargo run -p nve-cli -- --help
```

---

## 💡 Development Guidelines

### Code Style

- Follow Rust API Guidelines.
- Use `cargo fmt` to format your code before committing.
- Run `cargo clippy` to lint your code.
- Keep modules small and focused.
- Use `#[derive(Debug)]` on structs and enums for easier debugging.

### Commits

- Use clear, descriptive commit messages.
- Recommended format:

  ```bash
  <type>(<scope>): <description>
  ```

  Examples:
  - `feat(core): add remote command`
  - `fix(installer): handle missing home directory`
  - `refactor(layout): simplify current_dir path`

### Branch Naming

- `feature/<short-description>`
- `fix/<short-description>`
- `docs/<short-description>`

Examples:

- `feature/add-use-command`
- `fix/linux-tar-extraction`

---

## 📦 Testing Your Changes

Run all checks before submitting a PR:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

---

## 🔄 Submitting a Pull Request

1. Create a branch from `main`.
2. Make your changes and commit them.
3. Push your branch:

   ```bash
   git push origin feature/my-feature
   ```

4. Open a Pull Request (PR) to `main` in GitHub.
5. Ensure all CI checks pass.

---

## 🐛 Reporting Issues

If you find a bug:

- Search existing issues first.
- If no issue exists, open a new one with:
  - Steps to reproduce
  - Expected behavior
  - Actual behavior
  - Environment details (OS, architecture, Rust version)

---

## 🌟 Feature Requests

We welcome feature suggestions! Please open a Discussion or an Issue with:

- Problem you’re trying to solve
- Proposed solution
- Alternatives considered (if any)

---

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thanks for helping make **NVE** better! 🚀
