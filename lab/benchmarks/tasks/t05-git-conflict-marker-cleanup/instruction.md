`generated.conf` is consumed by `render_config.py`. It must be a valid INI file for the production profile:

- `host` is `127.0.0.1`
- `port` is `8443`
- `mode` is `production`
- `tls` is `required`
- `retries` is `3`
- audit is enabled with JSON output

`python3 render_config.py generated.conf` must print exactly `url=https://127.0.0.1:8443 mode=production audit=json retries=3`. Keep the parser behavior intact: incompatible production settings should still be rejected.
