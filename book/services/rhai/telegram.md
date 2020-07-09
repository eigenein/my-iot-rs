# [Telegram](../telegram.md) in [Rhai](../rhai.md)

## Example

```rust,noplaypen
let message = new_send_message(-1001349838037, "@eigenein Hello *World*");
message.parse_mode = parse_mode_markdown_v2;
telegram.call(message);
```
