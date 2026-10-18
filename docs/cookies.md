# Cookies

leetcode-cli talks to LeetCode as you, so it needs your session cookies. There are three ways to provide them.

## 1. Automatic (Chrome)

If `csrf` and `session` are both empty in `leetcode.toml`, leetcode-cli reads the cookies straight from **Chrome's** cookie store for the configured `site`. Just sign in to LeetCode in Chrome and run any command — there is nothing else to set up.

This is the only automatic path, and it is Chrome-only. It works on **macOS and Linux**; on **Windows** Chrome's cookie encryption isn't read automatically yet, so use the manual setup below. The same applies if you use a different browser, or Chrome's cookie store can't be read.

> If you see a "not logged in to Chrome" error, either sign in to LeetCode in Chrome or set the cookies manually.

## 2. Manual (any browser)

Copy the two cookie values into `leetcode.toml`:

```toml
[cookies]
csrf = '<your-leetcode-csrf-token>'
session = '<your-leetcode-session-key>'
site = 'leetcode.com'
```

To find them — for example in Firefox, after logging in to LeetCode:

1. Press <kbd>F12</kbd> and open the **Storage** tab.
2. Expand **Cookies** and select `https://leetcode.com`.
3. Copy the `Value` of `csrftoken` into `csrf` and `LEETCODE_SESSION` into `session`.

The same values are available under DevTools in any Chromium browser (Application → Cookies).

## 3. Environment variables

To keep secrets out of `leetcode.toml`, leave `csrf` and `session` empty and export them instead. Environment variables override whatever is in the file:

```sh
export LEETCODE_CSRF='<your-leetcode-csrf-token>'
export LEETCODE_SESSION='<your-leetcode-session-key>'
export LEETCODE_SITE='leetcode.com'   # or 'leetcode.cn'
```

`cookies.site` must still be present in `leetcode.toml` (otherwise config parsing fails), but `LEETCODE_SITE` overrides it at runtime.

## leetcode.com vs leetcode.cn

`site` accepts exactly two values: `leetcode.com` or `leetcode.cn` (anything else is rejected). Choosing `leetcode.cn` switches every API endpoint to the China site — set it via the config field or `LEETCODE_SITE`.

## Cloudflare challenges

The client enables HTTP/2 negotiation. Debug logging (`--debug`) reports the
negotiated protocol and response status, without logging cookie values.

When Chrome can run a solution but the CLI receives a Cloudflare challenge, set
`user_agent` under `[cookies]` in `~/.leetcode/leetcode.toml` to the exact
User-Agent from Chrome's successful request (DevTools → Network → Request Headers).

```toml
# Add these to the existing [cookies] section:
user_agent = 'Mozilla/5.0 (...)' # Replace with your browser's complete User-Agent
cf_clearance = ''
```

With automatic Chrome login cookies, `cf_clearance` is also read from Chrome when
available. A nonempty configured clearance overrides the Chrome value. With
manual login cookies, copy `cf_clearance` from the same browser into the config.
If Chrome has no clearance cookie, no clearance is sent.

Environment variables override the corresponding config fields:

```sh
export LEETCODE_USER_AGENT='<your-browser-user-agent>'
export LEETCODE_CF_CLEARANCE='<your-browser-cf_clearance>'
```

Use the same network connection as Chrome. Clearance expires and is tied to the
browser's identity; copying it and its User-Agent does not guarantee that an HTTP
client will be accepted. If challenged again, check the site in Chrome and refresh
the clearance. The CLI reports a Cloudflare-specific error rather than parsing the
challenge HTML as JSON.
