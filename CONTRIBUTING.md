<div align="center">

# contributing to lithium-engine

*thanks for helping improve lithium-engine!*

</div>

---

### Introduction

Contributions of all kinds are welcome: bug fixes, new features, documentation improvements, or even just trying things out and reporting issues.
Keep in mind the project is **work in progress**, so things may change quickly.

---

### Requirements

- [Rust](https://www.rust-lang.org/tools/install) (latest stable recommended)
- Git (to clone and manage branches)
- Linux (tested), Windows (tested), macOS (untested but should work)

---

### Getting started

1. clone the repository:
   ```bash
   git clone https://github.com/gabvigano/lithium.git
   cd lithium
   ```
2. run the demo game to make sure everything works:
   ```bash
   cd dropline
   cargo run
   ```

---

### Code style

- format with `cargo fmt` before committing
- lint with `cargo clippy` and fix warnings when possible
- keep commits small and messages **clear**. Use the following syntax:
   ```txt
   add collision detection and rendering to polygons:

   - implement sat check
   - fix hitbox overlap issue
   - add rendering of polygons
   ```
- use only **english** for everything
- write commits and comments in **lowercase**

---

### Workflow

1. create a new branch:
   ```bash
   git checkout -b feature/my-feature
   ```
2. make your changes and commit them:
   ```bash
   git commit -m "commit message"
   ```
3. push to your fork and open a Pull Request against `main`.

---

### Ways to contribute

- fix bugs
- improve performance
- add engine features (ecs, physics, rendering, ...)
- suggest new features by adding them to [todo.txt](./todo.txt)
- claim and implement features from [todo.txt](./todo.txt), including your own suggestions
- improve documentation & examples

please ensure:
- follow coding style (see above)
- update docs where relevant
- open a pull request with a clear description

---

### License

By contributing, you agree that your code will be licensed under the same terms as this repository.
See [LICENSE](./LICENSE.md) for details.
