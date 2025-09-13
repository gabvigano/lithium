<div align="center">

# 🤝 contributing to lithium-engine 🦀

*thanks for helping improve lithium-engine!*

</div>

---

### 📖 Introduction

Contributions of all kinds are welcome: bug fixes, new features, documentation improvements, or even just trying things out and reporting issues.
Keep in mind the project is **work in progress**, so things may change quickly.

---

### 📦 Requirements

- [Rust](https://www.rust-lang.org/tools/install) (latest stable recommended)
- Git (to clone and manage branches)
- Linux (tested), Windows (tested), macOS (untested but should work)

---

### 🚀 Getting started

1. Clone the repository:
   ```bash
   git clone https://github.com/gabvigano/lithium.git
   cd lithium
   ```
2. Run the demo game to make sure everything works:
   ```bash
   cd dropline
   cargo run
   ```

---

### ✍️ Code style

- Format with `cargo fmt` before committing.
- Lint with `cargo clippy` and fix warnings when possible.
- Keep commits small and messages clear. Use the following syntax:
   ```txt
   add collision detection and rendering to polygons:

   - implement sat check
   - fix hitbox overlap issue
   - add rendering of polygons
   ```
- Use only **english** for everything.
- Write commits and comments in **lowercase**.

---

### 🌱 Workflow

1. Create a new branch:
   ```bash
   git checkout -b feature/my-feature
   ```
2. Make your changes and commit them:
   ```bash
   git commit -m "commit message"
   ```
3. Push to your fork and open a Pull Request against `main`.

---

### 💡 What you can contribute

- 🐞 Bug fixes
- ⚡ Engine features (ECS, physics, rendering)
- 🔥 New features (listed in [todo.txt](./todo.txt))
- 🎮 Demo game improvements
- 📚 Documentation & examples

---

### 📜 License

By contributing, you agree that your code will be licensed under the same terms as this repository.
See [LICENSE](./LICENSE) for details.
