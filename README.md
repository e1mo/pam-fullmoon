# pam_fullmoon.so

Make pam debugging even more cursed!

## Installation & Configuration

First of all: no. If you realy are considering to use this, please refer to [Security](#security) beforehand!

Now that is out of the way, simply clone the repository as shown, compile the module so file and configure pam to use it.

```shell
# cd <path to pam-fullmoon> 
cargo build --release
sudo cp target/release/libpam_fullmoon.so /lib/security/pam_fullmoon.so
```

To actually use this module, it needs to be configured for a specific application. For testing (e.g. the `test.c` scrip) a separate `fullmoon` pam config seems like a sensible idea.

```
# /etc/pam.d/fullmoon
auth [ignore=ok] pam_fullmoon.so # **Read the note below!**
account required pam_fullmoon.so action=allow
```

Only the account functionality (`pam_sm_acct_mgmt(3)`) is implemented. The other functions will return `PAM_IGNORE` so that it won't mess with the rest of the pam stack. However, the `[ignore=ok]`
part in the first line will change that behaviour. Instead of just going along, PAM will treat any ignore result as a successfull authentication. **Placing this as is in your normal pam configs _will_
allow _everyone_ to log in!**

The optional `action`argument control weither users should only be able to log in when it's fullmoon (`action=allow`) or normalle be able to log in except when it is fullmoon (`action=deny`). Default is `deny`.  

## Security

I would not advise to use this in production. It probably is a half-decent gag if your colleauges / friends don't have anything to do right now. It hasn't been rigorously security tested by anyone. However, the codebase is quite small and I would judge it as being quite easy to follow along in order to find potential issues. That being said, take it for [the joke it is](https://chaos.social/@erikk/108850775052093856).

## Development / testing

When developing this module, simply create `/etc/pam.d/fullmoon`:

```text
auth [ignore=ok] /path/to/pam-fullmoon/target/debug/libpam_fullmoon.so
account required /path/to/pam-fullmoon/target/debug/libpam_fullmoon.so
```

By pointing this to the rust target directoy, you don't have to always copy the new `.so` file into the global directory after running `cargo build`.

To test this, compile and run the `test.c` file:

```shell
gcc -o target/pam_test -l pam -lpam_misc test.c
target/pam_test
```

## Acknowledgements

- The type definitions and function signatures were taken from [tailscale/pam](https://github.com/tailscale/pam), their testing scripts were also useful as someone wo had *no* experience with C.
- The `test.c` script comes from [anowell/pam-rs](https://github.com/anowell/pam-rs/blob/master/pam-sober/test.c).
