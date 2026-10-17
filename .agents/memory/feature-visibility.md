---
name: Compiler feature visibility
description: Why compiler upgrades need visible website entry points, not refresh advice alone.
---

Make new compiler capabilities easy to find on the playground and at the top of Reference; do not automatically replace the user's saved editor source to demonstrate an upgrade.

**Why:** After the tensor upgrade, the development API already served the updated compiler and example, but the unchanged layout/version label and instructions below a long manual made it appear unchanged. Repeated refresh advice did not resolve the user's confusion.

**How to apply:** Check the actual development-domain API before diagnosing stale caching. Distinguish the compiler's active capabilities from its package version and from locally saved source. Provide an explicit example-loading action and a clear documentation entry point.
