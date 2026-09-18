# Installation

## Prerequisites

Before installing jj-spr, you need to have Jujutsu (jj) installed on your system. See the [Jujutsu installation guide](https://github.com/jj-vcs/jj#installation) for instructions.

## Installation Methods

### From Source (Recommended)

jj-spr is written in Rust. You need a Rust toolchain to build from source. See [rustup.rs](https://rustup.rs) for information on how to install Rust if you don't have it already.

**Using cargo install (easiest):**

```shell
git clone https://github.com/LucioFranco/jj-spr.git
cd jj-spr
cargo install --path spr
```

This will install the `jj-spr` binary to your `~/.cargo/bin` directory.

## Verify Installation

After installation, verify that jj-spr is available:

```shell
jj-spr --version
```

## Next Steps

After installation, you'll need to:

1. **Set up the Jujutsu alias** to use jj-spr as a subcommand:
   ```shell
   jj config set --user aliases.spr '["util", "exec", "--", "jj-spr"]'
   ```

2. **Initialize in your repository**:
   ```shell
   cd your-jujutsu-repo
   jj spr init
   ```

See the [Setup Guide](./setup.md) for detailed configuration instructions.

## Certificate trust

The GitHub GraphQL client trusts both bundled public certificate authorities and
certificates from the operating system's trust store. This supports devboxes and
corporate networks with an internal certificate authority while retaining the
bundled public roots. TLS certificate verification remains enabled.

If you see `invalid peer certificate: UnknownIssuer`, make sure your network's
trusted CA is installed in the system trust store. On Linux, native certificate
loading also supports `SSL_CERT_FILE` and `SSL_CERT_DIR` for an existing trusted
CA bundle or directory.
