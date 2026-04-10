---
title: Unescaped less-than in a table
description: 'Demonstrates raw < in table cells'
---

# Latency table

This table uses a bare less-than before digits. That often confuses MDX or HTML pipelines.

| Metric   | Target |
| -------- | ------ |
| Latency  | <200ms |

Outside a table, the same issue applies: **target <10ms** for hot paths.

In fenced code it is fine:

```
if (x < 200) { ... }
```
