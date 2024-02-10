# A Rust Library for Managing CPU PL1/PL2 Power Limits
# Caution: MAY BREAK YOUR SYSTEM

Should only work on Windows x86 platforms, only tested on Intel Core Ultra7 155H, use at your own risk.

Requires `msr-utilities`, I don't know why I cannot make use of libwinring0 directly.

Reducing PL1/PL2 power limits can be useful for reducing power consumption and heat output, especially on laptops. But reducing it too much could result in:

1. A vvvvvery ssssssssslow system
2. Some device fails to work, including headphones, USB devices, etc.
3. Somehow CPU refuses to decode videos.
