---
title: Broken frontmatter quoting
description: "This string contains a "nested" quote without escaping"
---

# Body

The YAML above is invalid or ambiguous. Prefer:

```yaml
description: 'He said "yes" once.'
```

or escaped double quotes inside a double-quoted scalar.
