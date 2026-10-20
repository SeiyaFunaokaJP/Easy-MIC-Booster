# Contributing

Thanks for your interest in Easy MIC Booster!

Since this is a personal project, I appreciate any help or feedback.

## Reporting Bugs

If you find a bug or have a feature request, please **open an issue** on GitHub.
There are simple templates to help, but only the first box is required — a short note is perfectly fine, and English or Japanese both work.

## Sending Code

If you want to contribute code:

1.  Please refer to the **Developer Guide** for build instructions:
    - [Developer Guide (English)](docs/developer-guide.md)
    - [開発者ガイド (Japanese)](docs/ja/developer-guide.md)
2.  If you can, run these before sending (CI runs them as well):
    ```bash
    cargo fmt --all
    cargo clippy --all-targets
    cargo test
    python tools/check_lang.py
    ```
3.  Send a **Pull Request** with your changes.

Thank you!
