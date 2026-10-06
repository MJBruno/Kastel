# std.string

API utilitaire officielle autour de `str`.

| Fonction | Signature |
|---|---|
| `capitalize` | `str -> str` |
| `pad_left` | `(str, int, str) -> Result<str,str>` |
| `pad_right` | `(str, int, str) -> Result<str,str>` |
| `is_palindrome` | `str -> bool` |
| `parse_int` | `str -> Result<int,str>` |
| `parse_float` | `str -> Result<float,str>` |
| `find` | `(str,str) -> Option<int>` |
| `find_last` | `(str,str) -> Option<int>` |
| `char_at_opt` | `(str,int) -> Option<str>` |
| `truncate` | `(str,int,str) -> Result<str,str>` |
| `slug` | `str -> str` |
| `count_occurrences` | `(str,str) -> int` |

Les primitives du type `str` continuent de fournir `size()`, `byte_size()`,
`byte_at()`, `get()`, `slice()`, `split()`, `replace()`, etc.
